use anyhow::Result;
use clap::{Arg, Command};
use crossterm::{
    event::{self, Event, KeyEventKind, EnableBracketedPaste, DisableBracketedPaste},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dotenv::{dotenv, from_path};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::{env, io, path::PathBuf};
use tracing::info;

mod app;
mod ui;
mod theme;

use app::App;
use dialog_lib::{AppResult, Keys};

fn find_and_load_env() {
    // First try the standard dotenv() which looks for .env in current dir
    dotenv().ok();

    // Then walk up the directory tree looking for .env.local
    let mut current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    loop {
        let env_file = current_dir.join(".env.local");
        if env_file.exists() {
            from_path(&env_file).ok();
            break;
        }

        // Move up one directory
        match current_dir.parent() {
            Some(parent) => current_dir = parent.to_path_buf(),
            None => break, // Reached filesystem root
        }
    }
}

fn get_secret_key(key_arg: &str) -> Result<String> {
    match key_arg {
        "bob" => {
            find_and_load_env();
            env::var("BOB_SK_HEX")
                .map_err(|_| anyhow::anyhow!("BOB_SK_HEX not found in environment variables"))
        }
        "alice" => {
            find_and_load_env();
            env::var("ALICE_SK_HEX")
                .map_err(|_| anyhow::anyhow!("ALICE_SK_HEX not found in environment variables"))
        }
        hex_key => {
            // Validate that it looks like a hex string
            if hex_key.len() == 64 && hex_key.chars().all(|c| c.is_ascii_hexdigit()) {
                Ok(hex_key.to_string())
            } else {
                Err(anyhow::anyhow!("Key must be either 'bob', 'alice', or a 64-character hex string"))
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Set up logging
    tracing_subscriber::fmt()
        .with_env_filter("dialog_tui=debug")
        .init();

    // Set up command line argument parsing
    let matches = Command::new("dialog_tui")
        .version("0.1.0")
        .about("Terminal UI for Dialog messaging")
        .arg(
            Arg::new("key")
                .long("key")
                .value_name("KEY")
                .help("Secret key for identity: 'bob', 'alice', or hex string")
                .required(true),
        )
        .get_matches();

    let key_arg = matches.get_one::<String>("key").unwrap();
    let sk_hex = get_secret_key(key_arg)?;
    let keys = Keys::parse(&sk_hex)
        .map_err(|e| anyhow::anyhow!("Failed to parse secret key: {}", e))?;

    info!("Starting Dialog TUI with MLS operations for key: {}", key_arg);

    // Setup terminal
    enable_raw_mode().map_err(|e| anyhow::anyhow!("Failed to enable raw mode: {}", e))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableBracketedPaste)
        .map_err(|e| anyhow::anyhow!("Failed to setup terminal: {}", e))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| anyhow::anyhow!("Failed to create terminal: {}", e))?;

    // Create app with MLS service using provided keys
    let dialog_lib = dialog_lib::DialogLib::new_with_keys(keys).await
        .map_err(|e| anyhow::anyhow!("Failed to initialize MLS service: {}", e))?;
    
    let mut app = App::new_with_service(dialog_lib).await
        .map_err(|e| anyhow::anyhow!("Failed to create app: {}", e))?;

    // Run app
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableBracketedPaste
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

async fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        // Check for delayed messages and track if we got any
        let had_delayed_messages = app.check_delayed_messages();
        
        // Check for UI updates (real-time messages)
        let had_ui_updates = app.check_ui_updates().await;
        
        terminal.draw(|f| ui::draw(f, app))?;

        // Check for input events without blocking
        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    match app.handle_key(key).await {
                        AppResult::Continue => {}
                        AppResult::Exit => return Ok(()),
                    }
                }
                Event::Paste(data) => {
                    app.handle_paste(&data);
                }
                _ => {}
            }
        }
        
        // Small delay to prevent excessive CPU usage when no events
        // If we had delayed messages or UI updates, reduce the delay to make UI more responsive
        let delay = if had_delayed_messages || had_ui_updates {
            tokio::time::Duration::from_millis(16) // ~60fps when messages are arriving
        } else {
            tokio::time::Duration::from_millis(16) // Keep consistent ~60fps
        };
        tokio::time::sleep(delay).await;
    }
}
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
use dialog_lib::StorageBackend;

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

fn get_data_dir() -> Result<PathBuf> {
    // Try to get platform-specific data directory
    if let Some(data_dir) = dirs::data_dir() {
        Ok(data_dir.join("dialog"))
    } else if let Ok(home) = env::var("HOME") {
        // Fallback to home directory
        Ok(PathBuf::from(home).join(".dialog"))
    } else {
        // Last resort - current directory
        Ok(PathBuf::from(".dialog"))
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
        .arg(
            Arg::new("ephemeral")
                .long("ephemeral")
                .help("Use ephemeral (in-memory) storage instead of persistent SQLite")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let key_arg = matches.get_one::<String>("key").unwrap();
    let sk_hex = get_secret_key(key_arg)?;
    let keys = Keys::parse(&sk_hex)
        .map_err(|e| anyhow::anyhow!("Failed to parse secret key: {}", e))?;
    
    let use_ephemeral = matches.get_flag("ephemeral");

    info!("Starting Dialog TUI with MLS operations for key: {}", key_arg);

    // Setup terminal
    enable_raw_mode().map_err(|e| anyhow::anyhow!("Failed to enable raw mode: {}", e))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableBracketedPaste)
        .map_err(|e| anyhow::anyhow!("Failed to setup terminal: {}", e))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| anyhow::anyhow!("Failed to create terminal: {}", e))?;

    // Create app with MLS service using provided keys and storage backend
    let dialog_lib = if use_ephemeral {
        info!("Using ephemeral (memory) storage");
        dialog_lib::DialogLib::new_with_keys(keys).await
            .map_err(|e| anyhow::anyhow!("Failed to initialize MLS service: {}", e))?
    } else {
        let data_dir = get_data_dir()?;
        let db_path = data_dir.join(format!("{}.db", key_arg));
        info!("Using SQLite storage at: {:?}", db_path);
        
        // Get relay URL from config
        let config = dialog_lib::DialogConfig::new();
        let relay_url = config.relay_urls.first()
            .ok_or_else(|| anyhow::anyhow!("No relay URLs configured"))?
            .clone();
        
        dialog_lib::DialogLib::new_with_storage(
            keys,
            relay_url,
            StorageBackend::Sqlite { path: db_path }
        ).await
        .map_err(|e| anyhow::anyhow!("Failed to initialize MLS service with SQLite: {}", e))?
    };
    
    let mut app = App::new_with_service(dialog_lib).await
        .map_err(|e| anyhow::anyhow!("Failed to create app: {}", e))?;
    
    // Autoconnect on startup
    app.add_message("");
    app.add_message("⚡ Attempting to connect to relay...");
    match app.dialog_lib.toggle_connection().await {
        Ok(status) => {
            app.connection_status = status;
            if status == dialog_lib::ConnectionStatus::Connected {
                app.add_message("✅ Connected to relay successfully!");
                
                // Create new channel for UI updates
                let (ui_update_tx, ui_update_rx) = tokio::sync::mpsc::channel(100);
                app.ui_update_rx = Some(ui_update_rx);
                
                // Start subscription for real-time messages
                if let Err(e) = app.dialog_lib.subscribe_to_groups(ui_update_tx).await {
                    app.add_message(&format!("⚠️  Failed to start real-time message subscription: {}", e));
                } else {
                    app.add_message("✅ Real-time message updates enabled");
                }
                
                // Refresh data to get latest state
                app.refresh_data().await;
                
                // Publish key packages on startup
                app.add_message("");
                if use_ephemeral {
                    app.add_message("🔐 Publishing fresh key packages (ephemeral mode)...");
                } else {
                    app.add_message("🔐 Publishing key packages...");
                }
                match app.dialog_lib.publish_key_packages().await {
                    Ok(event_ids) => {
                        app.add_message(&format!("✅ Published {} key packages", event_ids.len()));
                        
                        // Show event IDs for observability
                        app.add_message("📋 Key package event IDs:");
                        for (i, event_id) in event_ids.iter().enumerate() {
                            app.add_message(&format!("    {}: {}...{}", 
                                i + 1, 
                                &event_id[0..8], 
                                &event_id[event_id.len()-8..]
                            ));
                        }
                        
                        app.add_message("");
                        if use_ephemeral {
                            app.add_message("⚠️  EPHEMERAL MODE: You can only accept invites sent during THIS session");
                            app.add_message("    (Memory storage means HPKE keys are lost on restart)");
                        } else {
                            app.add_message("✅ PERSISTENT MODE: Your keys are saved and will work across restarts");
                            app.add_message("    (SQLite storage preserves HPKE keys and forward secrecy)");
                        }
                    }
                    Err(e) => {
                        app.add_message(&format!("❌ Failed to publish key packages: {}", e));
                        app.add_message("⚠️  You won't be able to receive group invites!");
                        app.add_message("    Try /keypackage to publish manually");
                    }
                }
            } else {
                app.add_message("❌ Failed to connect to relay");
                app.add_message("You can try /connect later to establish a connection");
            }
        }
        Err(e) => {
            app.add_message(&format!("❌ Connection failed: {}", e));
            app.add_message("You can try /connect later to establish a connection");
        }
    }
    app.add_message("");

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
        let delay = if had_ui_updates {
            tokio::time::Duration::from_millis(16) // ~60fps when messages are arriving
        } else {
            tokio::time::Duration::from_millis(16) // Keep consistent ~60fps
        };
        tokio::time::sleep(delay).await;
    }
}
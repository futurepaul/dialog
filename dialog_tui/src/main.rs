use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyEventKind, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use tracing::info;

mod app;
mod ui;
mod theme;

use app::{App, AppResult};

#[tokio::main]
async fn main() -> Result<()> {
    // Set up logging
    tracing_subscriber::fmt()
        .with_env_filter("dialog_tui=debug")
        .init();

    info!("Starting Dialog TUI with fullscreen mode");

    // Setup terminal
    enable_raw_mode().map_err(|e| anyhow::anyhow!("Failed to enable raw mode: {}", e))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .map_err(|e| anyhow::anyhow!("Failed to setup terminal: {}", e))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| anyhow::anyhow!("Failed to create terminal: {}", e))?;

    // Create app
    let mut app = App::new();

    // Run app
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

async fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        // Check for delayed messages before drawing
        app.check_delayed_messages();
        
        terminal.draw(|f| ui::draw(f, app))?;

        // Use timeout to ensure we process delayed messages even when no input
        if let Ok(event) = tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            async { event::read() }
        ).await {
            if let Ok(Event::Key(key)) = event {
                if key.kind == KeyEventKind::Press {
                    match app.handle_key(key).await {
                        AppResult::Continue => {}
                        AppResult::Exit => return Ok(()),
                    }
                }
            }
        }
    }
}
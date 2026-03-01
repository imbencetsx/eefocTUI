mod app;
mod config;
mod console;
mod events;
mod models;
mod services;
mod ui;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    let (tx, rx) = mpsc::unbounded_channel();

    // Load config (currently defaults only).
    let config = config::Config::default();

    // App state
    let mut app = App::new(config.clone(), tx.clone());

    // Spawn background tasks
    services::spawn_background_tasks(&app, tx.clone());

    // Spawn input loop (keyboard + ticks)
    let tick_rate = std::time::Duration::from_millis(config.tick_rate_ms);
    tokio::spawn(events::input::input_loop(tx, tick_rate));

    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let res = ui::run_app(&mut terminal, &mut app, rx).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {err:?}");
    }

    Ok(())
}


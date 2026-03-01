pub mod icons;
pub mod layout;
pub mod theme;
pub mod views;

use crate::app::App;
use crate::events::AppEvent;
use ratatui::Terminal;
use ratatui::backend::Backend;
use tokio::sync::mpsc::UnboundedReceiver;

pub async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    mut rx: UnboundedReceiver<AppEvent>,
) -> Result<(), B::Error> {
    while !app.should_quit {
        if let Some(event) = rx.recv().await {
            match event {
                AppEvent::Quit => {
                    app.should_quit = true;
                }
                other => app.update(other),
            }
        }

        terminal.draw(|frame| {
            views::render(frame, app);
        })?;
    }

    Ok(())
}


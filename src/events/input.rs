use crate::console;
use crate::events::AppEvent;
use crossterm::event::{self, Event as CEvent, KeyCode, KeyEventKind, KeyModifiers};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::UnboundedSender;

static NETWORK_INPUT_ACTIVE: AtomicBool = AtomicBool::new(false);

pub fn is_network_input_active() -> bool {
    NETWORK_INPUT_ACTIVE.load(Ordering::SeqCst)
}

pub fn set_network_input_active(active: bool) {
    NETWORK_INPUT_ACTIVE.store(active, Ordering::SeqCst);
}

pub async fn input_loop(tx: UnboundedSender<AppEvent>, tick_rate: Duration) {
    let mut last_tick = Instant::now();

    loop {
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_millis(0));

        if event::poll(timeout).unwrap_or(false) {
            if let Ok(CEvent::Key(key)) = event::read() {
                if let KeyEventKind::Press = key.kind {
                    let ev = if console::is_console_active() {
                        handle_console_input(key)
                    } else if is_network_input_active() {
                        handle_network_input(key)
                    } else {
                        handle_main_input(key)
                    };
                    if let Some(ev) = ev {
                        if tx.send(ev).is_err() {
                            return;
                        }
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
            if tx.send(AppEvent::Tick).is_err() {
                return;
            }
        }
    }
}

fn handle_console_input(key: event::KeyEvent) -> Option<AppEvent> {
    match (key.code, key.modifiers) {
        (KeyCode::Esc, _) => Some(AppEvent::ClearError),
        (KeyCode::Enter, _) => Some(AppEvent::DockerConsoleEnter),
        (KeyCode::Insert, _) => Some(AppEvent::DockerConsoleDetach),
        (KeyCode::Backspace, _) => Some(AppEvent::DockerConsoleBackspace),
        (KeyCode::PageUp, _) => Some(AppEvent::DockerConsoleScrollUp),
        (KeyCode::PageDown, _) => Some(AppEvent::DockerConsoleScrollDown),
        (KeyCode::Up, KeyModifiers::ALT) => Some(AppEvent::DockerTableScrollUp),
        (KeyCode::Down, KeyModifiers::ALT) => Some(AppEvent::DockerTableScrollDown),
        (KeyCode::Up, _) => Some(AppEvent::DockerConsoleScrollUp),
        (KeyCode::Down, _) => Some(AppEvent::DockerConsoleScrollDown),
        (KeyCode::Left, _) => Some(AppEvent::DockerConsoleCursorLeft),
        (KeyCode::Right, _) => Some(AppEvent::DockerConsoleCursorRight),
        (KeyCode::End, _) => Some(AppEvent::DockerConsoleScrollToBottom),
        (KeyCode::Char(ch), _) => Some(AppEvent::DockerConsoleKey(ch)),
        _ => None,
    }
}

fn handle_network_input(key: event::KeyEvent) -> Option<AppEvent> {
    match (key.code, key.modifiers) {
        (KeyCode::Esc, _) => {
            set_network_input_active(false);
            Some(AppEvent::NetworkInputExit)
        }
        (KeyCode::Left, KeyModifiers::CONTROL) => Some(AppEvent::PrevView),
        (KeyCode::Right, KeyModifiers::CONTROL) => Some(AppEvent::NextView),
        (KeyCode::Left, _) => Some(AppEvent::NetworkPortInputCursorLeft),
        (KeyCode::Right, _) => Some(AppEvent::NetworkPortInputCursorRight),
        (KeyCode::Tab, KeyModifiers::NONE) => Some(AppEvent::NetworkPortInputTab),
        (KeyCode::Char('p'), _) => Some(AppEvent::NetworkCyclePreset),
        (KeyCode::Enter, _) => Some(AppEvent::NetworkPortScanStart),
        (KeyCode::Backspace, _) => Some(AppEvent::NetworkPortInputBackspace),
        (KeyCode::Char(ch), _) => Some(AppEvent::NetworkPortInputKey(ch)),
        _ => None,
    }
}

fn handle_main_input(key: event::KeyEvent) -> Option<AppEvent> {
    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), _) => Some(AppEvent::Quit),
        (KeyCode::Char('h'), _) | (KeyCode::Left, KeyModifiers::CONTROL) => {
            Some(AppEvent::PrevView)
        }
        (KeyCode::Char('l'), _) | (KeyCode::Right, KeyModifiers::CONTROL) => {
            Some(AppEvent::NextView)
        }
        (KeyCode::Up, KeyModifiers::ALT) => Some(AppEvent::DockerTableScrollUp),
        (KeyCode::Down, KeyModifiers::ALT) => Some(AppEvent::DockerTableScrollDown),
        (KeyCode::Up, _) => Some(AppEvent::MoveSelectionUp),
        (KeyCode::Down, _) => Some(AppEvent::MoveSelectionDown),
        // Docker view actions
        (KeyCode::Char('o'), _) => Some(AppEvent::DockerStartSelected),
        (KeyCode::Char('s'), _) => Some(AppEvent::DockerStopSelected),
        (KeyCode::Char('r'), _) => Some(AppEvent::DockerRestartSelected),
        (KeyCode::Delete, _) => Some(AppEvent::DockerDeleteKey),
        (KeyCode::Enter, _) => Some(AppEvent::DockerAttachSelected),
        // Network view
        (KeyCode::Tab, KeyModifiers::NONE) => Some(AppEvent::NetworkNextSubView),
        (KeyCode::BackTab, _) => Some(AppEvent::NetworkPrevSubView),
        (KeyCode::Char('i'), _) => {
            set_network_input_active(true);
            Some(AppEvent::NetworkInputEnter)
        }
        // Esc clears transient UI
        (KeyCode::Esc, _) => Some(AppEvent::ClearError),
        _ => None,
    }
}

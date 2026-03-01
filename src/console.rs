use std::sync::atomic::{AtomicBool, Ordering};

/// Global flag indicating whether the in-app Docker console is currently active.
///
/// This is used by the input loop (which doesn't have direct access to `App`)
/// to decide whether to route key presses to normal shortcuts (views, start/stop
/// containers, etc.) or to the embedded console for the selected container.
static CONSOLE_ACTIVE: AtomicBool = AtomicBool::new(false);

pub fn set_console_active(active: bool) {
    CONSOLE_ACTIVE.store(active, Ordering::SeqCst);
}

pub fn is_console_active() -> bool {
    CONSOLE_ACTIVE.load(Ordering::SeqCst)
}


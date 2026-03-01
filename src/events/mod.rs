pub mod input;

use crate::app::{NetworkConnection, NetworkInterface};
use crate::models::{container::Container, metrics::SystemMetrics};

#[derive(Debug)]
pub enum AppEvent {
    // Core app control
    Tick,
    Quit,

    // Navigation
    NextView,
    PrevView,
    MoveSelectionUp,
    MoveSelectionDown,

    // Docker actions (from keyboard)
    DockerStartSelected,
    DockerStopSelected,
    DockerRestartSelected,
    DockerDeleteKey,
    DockerAttachSelected,

    // Docker in-app console
    DockerConsoleReset(Vec<String>),
    DockerConsoleOutput(String),
    DockerConsoleClosed,
    DockerConsoleKey(char),
    DockerConsoleBackspace,
    DockerConsoleCursorLeft,
    DockerConsoleCursorRight,
    DockerConsoleEnter,
    DockerConsoleDetach,
    DockerConsoleScrollUp,
    DockerConsoleScrollDown,
    DockerConsoleScrollToBottom,

    // Docker data
    DockerContainersUpdated(Vec<Container>),
    DockerError(String),

    // Docker table scroll
    DockerTableScrollUp,
    DockerTableScrollDown,

    // System metrics
    SystemMetricsUpdated(SystemMetrics),

    // Network view
    NetworkUpdated(Vec<NetworkInterface>, Vec<NetworkConnection>),
    NetworkNextSubView,
    NetworkPrevSubView,
    NetworkInputEnter,
    NetworkInputExit,
    NetworkCyclePreset,
    NetworkPortInputKey(char),
    NetworkPortInputBackspace,
    NetworkPortInputCursorLeft,
    NetworkPortInputCursorRight,
    NetworkPortInputTab,
    NetworkPortScanStart,
    NetworkPortScanResult(u16, bool, String),

    // UI helpers
    ClearError,
}


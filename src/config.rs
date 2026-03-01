use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// UI tick rate in milliseconds (controls animation smoothness).
    pub tick_rate_ms: u64,
    /// System metrics refresh interval in milliseconds.
    pub metrics_interval_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tick_rate_ms: 120,
            metrics_interval_ms: 1_000,
        }
    }
}


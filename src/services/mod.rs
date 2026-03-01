pub mod docker;
pub mod network;
pub mod system;

use crate::app::App;
use crate::config::Config;
use crate::events::AppEvent;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

pub fn spawn_background_tasks(app: &App, tx: UnboundedSender<AppEvent>) {
    let config: Config = app.config.clone();

    // Docker polling
    {
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            docker::poll_docker_loop(tx_clone, Duration::from_secs(3)).await;
        });
    }

    // System metrics polling
    {
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            system::poll_system_metrics_loop(
                tx_clone,
                Duration::from_millis(config.metrics_interval_ms),
            )
            .await;
        });
    }

    // Network polling
    {
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            network::poll_network_loop(tx_clone, Duration::from_secs(5)).await;
        });
    }
}


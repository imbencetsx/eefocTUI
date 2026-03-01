use crate::events::AppEvent;
use crate::models::metrics::{
    CpuCoreUsage, DiskUsage, MemoryUsage, NetworkUsage, SystemMetrics,
};
use std::time::Duration;
use sysinfo::{Disks, NetworkData, Networks, System};
use tokio::sync::mpsc::UnboundedSender;
use tokio::time;

pub async fn poll_system_metrics_loop(tx: UnboundedSender<AppEvent>, interval: Duration) {
    let mut sys = System::new_all();
    let mut disks = Disks::new_with_refreshed_list();
    let mut networks = Networks::new_with_refreshed_list();

    loop {
        // Refresh core system information.
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        disks.refresh(true);
        networks.refresh(true);

        let total_cpu = sys.global_cpu_usage();
        let cpu_cores = sys
            .cpus()
            .iter()
            .enumerate()
            .map(|(i, cpu)| CpuCoreUsage {
                name: format!("CPU {i}"),
                usage: cpu.cpu_usage(),
            })
            .collect();

        let total_memory = sys.total_memory();
        let used_memory = sys.used_memory();
        let memory = MemoryUsage {
            used: used_memory,
            total: total_memory,
        };

        let mut disk_list: Vec<DiskUsage> = disks
            .list()
            .iter()
            .map(|d| DiskUsage {
                name: d.name().to_string_lossy().to_string(),
                mount_point: d.mount_point().to_string_lossy().to_string(),
                used: d.total_space().saturating_sub(d.available_space()),
                total: d.total_space(),
            })
            .collect();
        // De-duplicate by mount point (helps avoid repeated "overlay" entries).
        disk_list.sort_by(|a, b| a.mount_point.cmp(&b.mount_point));
        disk_list.dedup_by(|a, b| a.mount_point == b.mount_point);

        let mut rx_total: u64 = 0;
        let mut tx_total: u64 = 0;
        for (_if_name, data) in networks.iter() {
            let data: &NetworkData = data;
            rx_total = rx_total.saturating_add(data.received());
            tx_total = tx_total.saturating_add(data.transmitted());
        }
        let network = NetworkUsage {
            received: rx_total,
            transmitted: tx_total,
        };

        let metrics = SystemMetrics {
            cpu_total: total_cpu,
            cpu_cores,
            memory,
            disks: disk_list,
            network,
        };

        let _ = tx.send(AppEvent::SystemMetricsUpdated(metrics));

        time::sleep(interval).await;
    }
}


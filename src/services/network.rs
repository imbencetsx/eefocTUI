use crate::app::{NetworkConnection, NetworkInterface};
use crate::events::AppEvent;
use anyhow::Result;
use std::time::Duration;
use tokio::process::Command;
use tokio::sync::mpsc::UnboundedSender;

pub async fn poll_network_loop(tx: UnboundedSender<AppEvent>, interval: Duration) {
    loop {
        if let Err(err) = refresh_network(&tx).await {
            let _ = tx.send(AppEvent::DockerError(format!("Network error: {err:#}")));
        }
        tokio::time::sleep(interval).await;
    }
}

pub async fn refresh_network(tx: &UnboundedSender<AppEvent>) -> Result<()> {
    let interfaces = get_network_interfaces().await.unwrap_or_default();
    let connections = get_network_connections().await.unwrap_or_default();
    
    let _ = tx.send(AppEvent::NetworkUpdated(interfaces, connections));
    Ok(())
}

async fn get_network_interfaces() -> Result<Vec<NetworkInterface>> {
    let mut interfaces: Vec<NetworkInterface> = Vec::new();

    #[cfg(unix)]
    {
        let output = Command::new("ip")
            .args(["-br", "addr", "show"])
            .output()
            .await?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name = parts[0].to_string();
                    let status = parts.get(1).unwrap_or(&"").to_string();
                    let is_up = status == "UP" || status.to_lowercase().contains("up");
                    
                    let ip = parts.get(2)
                        .map(|s| s.split('/').next().unwrap_or("").to_string())
                        .unwrap_or_default();
                    
                    interfaces.push(NetworkInterface {
                        name,
                        ip,
                        mac: String::new(),
                        is_up,
                    });
                }
            }
        }

        for iface in &mut interfaces {
            let mac_output = Command::new("cat")
                .args([&format!("/sys/class/net/{}/address", iface.name)])
                .output()
                .await;
            
            if let Ok(out) = mac_output {
                if out.status.success() {
                    iface.mac = String::from_utf8_lossy(&out.stdout).trim().to_string();
                }
            }
        }
    }

    #[cfg(not(unix))]
    {
        let output = Command::new("ipconfig")
            .output()
            .await?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut current_name = String::new();
            let mut current_ip = String::new();
            let mut current_mac = String::new();
            let mut is_up = false;

            for line in stdout.lines() {
                let line = line.trim();
                if line.ends_with(':') && !line.starts_with(' ') {
                    if !current_name.is_empty() {
                        interfaces.push(NetworkInterface {
                            name: current_name.clone(),
                            ip: current_ip.clone(),
                            mac: current_mac.clone(),
                            is_up,
                        });
                    }
                    current_name = line.trim_end_matches(':').to_string();
                    current_ip = String::new();
                    current_mac = String::new();
                    is_up = false;
                } else if line.contains("IPv4 Address") || line.contains("IPv4") {
                    if let Some(ip) = line.split(':').last() {
                        current_ip = ip.trim().to_string();
                        is_up = true;
                    }
                } else if line.contains("Physical Address") || line.contains("MAC") {
                    if let Some(mac) = line.split(':').last() {
                        current_mac = mac.trim().to_string();
                    }
                }
            }

            if !current_name.is_empty() {
                interfaces.push(NetworkInterface {
                    name: current_name,
                    ip: current_ip,
                    mac: current_mac,
                    is_up,
                });
            }
        }
    }

    Ok(interfaces)
}

async fn get_network_connections() -> Result<Vec<NetworkConnection>> {
    let mut connections: Vec<NetworkConnection> = Vec::new();

    #[cfg(unix)]
    {
        let output = Command::new("ss")
            .args(["-tunap"])
            .output()
            .await?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    let protocol = if parts[0].contains('t') { "TCP" } else { "UDP" };
                    let local_addr = parts.get(4).unwrap_or(&"").to_string();
                    let remote_addr = parts.get(5).unwrap_or(&"*").to_string();
                    let state = parts.get(1).unwrap_or(&"").to_string();
                    let pid = parts.last()
                        .and_then(|s| s.split(',').next())
                        .and_then(|s| s.parse::<u32>().ok())
                        .unwrap_or(0);

                    connections.push(NetworkConnection {
                        protocol: protocol.to_string(),
                        local_addr,
                        remote_addr,
                        state,
                        pid,
                    });
                }
            }
        }
    }

    #[cfg(not(unix))]
    {
        let output = Command::new("netstat")
            .args(["-ano"])
            .output()
            .await?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(4) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let protocol = parts.get(0).unwrap_or(&"").to_string();
                    let local_addr = parts.get(1).unwrap_or(&"").to_string();
                    let remote_addr = parts.get(2).unwrap_or(&"").to_string();
                    let state = parts.get(3).unwrap_or(&"").to_string();
                    let pid = parts.last()
                        .and_then(|s| s.parse::<u32>().ok())
                        .unwrap_or(0);

                    connections.push(NetworkConnection {
                        protocol,
                        local_addr,
                        remote_addr,
                        state,
                        pid,
                    });
                }
            }
        }
    }

    Ok(connections)
}

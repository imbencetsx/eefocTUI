use crate::events::AppEvent;
use crate::models::container::Container;
use anyhow::Result;
use bollard::Docker;
use bollard::query_parameters::{
    ListContainersOptions, RemoveContainerOptions, RestartContainerOptions, StartContainerOptions,
    StopContainerOptions,
};
use std::collections::HashMap;
use std::time::Duration;
use tokio::process::Command;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time;

/// Try to fetch live Docker stats for all containers using the `docker` CLI.
///
/// This keeps the Bollard integration simple while still giving us
/// realistic CPU% and memory usage numbers that match `docker stats`.
async fn fetch_docker_stats_cli() -> HashMap<String, (f64, String)> {
    let mut map = HashMap::new();

    let output = Command::new("docker")
        .args([
            "stats",
            "--no-stream",
            "--format",
            "{{.ID}}\t{{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}",
        ])
        .output()
        .await;

    let output = match output {
        Ok(out) if out.status.success() => out,
        _ => {
            // If the CLI is unavailable or stats fail, we simply fall back
            // to zeros/"n/a" for the UI instead of surfacing an error.
            return map;
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let parts: Vec<_> = line.split('\t').collect();
        if parts.len() < 4 {
            continue;
        }

        let id = parts[0].trim().to_string();
        let name = parts[1].trim().to_string();
        let cpu_raw = parts[2].trim().trim_end_matches('%').trim();
        // Some locales use comma as decimal separator, normalise to dot.
        let cpu_raw_norm = cpu_raw.replace(',', ".");
        let cpu_percent = cpu_raw_norm.parse::<f64>().unwrap_or(0.0);
        let mem_usage = parts[3].trim().to_string();

        if !id.is_empty() {
            map.insert(id.clone(), (cpu_percent, mem_usage.clone()));
        }

        if !name.is_empty() {
            // Docker names are usually without the leading '/', but the
            // API sometimes reports them with one – normalise both.
            let normalised = name.trim_start_matches('/').to_string();
            map.insert(normalised, (cpu_percent, mem_usage.clone()));
        }
    }

    map
}

fn keep_ansi_line(s: &str) -> String {
    s.trim_end_matches(&['\r', '\n'][..]).to_string()
}

fn derive_uptime_from_status(status: &str) -> String {
    if let Some(pos) = status.find("Up ") {
        status[pos + 3..].trim().to_string()
    } else {
        "n/a".to_string()
    }
}

pub async fn poll_docker_loop(tx: UnboundedSender<AppEvent>, interval: Duration) {
    loop {
        if let Err(err) = refresh_containers(&tx).await {
            let _ = tx.send(AppEvent::DockerError(format!("{err:#}")));
        }
        time::sleep(interval).await;
    }
}

/// Fetch the current list of containers and push it into app state.
pub async fn refresh_containers(tx: &UnboundedSender<AppEvent>) -> Result<()> {
    let docker = Docker::connect_with_local_defaults()?;
    let stats_map = fetch_docker_stats_cli().await;
    let options = Some(ListContainersOptions {
        all: true,
        filters: Some(HashMap::new()),
        ..Default::default()
    });

    let list = docker.list_containers(options).await?;

    let containers: Vec<Container> = list
        .into_iter()
        .map(|c| {
            let id = c.id.unwrap_or_default();
            let raw_name = c
                .names
                .unwrap_or_default()
                .get(0)
                .cloned()
                .unwrap_or_else(|| id.clone());
            let // Docker API usually prefixes names with '/', but we prefer a clean name in the UI.
            name = raw_name.trim_start_matches('/').to_string();
            let image = c.image.unwrap_or_default();
            // Prefer Docker's textual "status" field; fall back to a generic label.
            let status = c.status.unwrap_or_else(|| "unknown".to_string());

            // Look up stats by ID first, then fall back to the (normalised) name.
            let (cpu_percent, memory) = stats_map
                .get(&id)
                .or_else(|| stats_map.get(&name))
                .cloned()
                .unwrap_or_else(|| (0.0, String::from("n/a")));

            let uptime = derive_uptime_from_status(&status);

            Container {
                id,
                name,
                image,
                status,
                cpu_percent,
                memory,
                uptime,
            }
        })
        .collect();

    let _ = tx.send(AppEvent::DockerContainersUpdated(containers));
    Ok(())
}

pub async fn start_container(id: &str) -> Result<()> {
    let docker = Docker::connect_with_local_defaults()?;
    // Default options are fine for start/stop/restart in this TUI.
    docker.start_container(id, None::<StartContainerOptions>).await?;
    Ok(())
}

pub async fn stop_container(id: &str) -> Result<()> {
    let docker = Docker::connect_with_local_defaults()?;
    docker
        .stop_container(id, None::<StopContainerOptions>)
        .await?;
    Ok(())
}

pub async fn restart_container(id: &str) -> Result<()> {
    let docker = Docker::connect_with_local_defaults()?;
    docker
        .restart_container(id, None::<RestartContainerOptions>)
        .await?;
    Ok(())
}

pub async fn delete_container(id: &str) -> Result<()> {
    let docker = Docker::connect_with_local_defaults()?;
    docker
        .remove_container(id, None::<RemoveContainerOptions>)
        .await?;
    Ok(())
}

// ---------------- Plain pipe-based console (non-unix fallback) ----------------

#[cfg(not(unix))]
use std::sync::OnceLock;

#[cfg(not(unix))]
use tokio::process::ChildStdin;

#[cfg(not(unix))]
use tokio::sync::Mutex;

/// Piped stdin for the current `docker attach` session (if any).
#[cfg(not(unix))]
static CONSOLE_STDIN: OnceLock<Mutex<Option<ChildStdin>>> = OnceLock::new();

#[cfg(not(unix))]
fn console_stdin() -> &'static Mutex<Option<ChildStdin>> {
    CONSOLE_STDIN.get_or_init(|| Mutex::new(None))
}

#[cfg(not(unix))]
async fn open_console_pipes(tx: &UnboundedSender<AppEvent>, id: &str) -> Result<()> {
    // 1) Fetch recent logs.
    let logs_output = Command::new("docker")
        .args(["logs", "--tail", "200", id])
        .output()
        .await?;

    if logs_output.status.success() {
        let stdout = String::from_utf8_lossy(&logs_output.stdout);
        let mut lines: Vec<String> = stdout.lines().map(|s| keep_ansi_line(s)).collect();
        const MAX_LINES: usize = 2000;
        if lines.len() > MAX_LINES {
            let excess = lines.len() - MAX_LINES;
            lines.drain(0..excess);
        }
        let _ = tx.send(AppEvent::DockerConsoleReset(lines));
    } else {
        let _ = tx.send(AppEvent::DockerConsoleReset(vec![format!(
            "(failed to fetch logs: {})",
            logs_output.status
        )]));
    }

    // 2) Start `docker attach` and wire it to events.
    let mut child = Command::new("docker")
        .args(["attach", "--sig-proxy=false", id])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let stdin = child.stdin.take();
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    if let Some(stdin) = stdin {
        *console_stdin().lock().await = Some(stdin);
    }

    if let Some(stdout) = stdout {
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let line = keep_ansi_line(&line);
                if tx_clone.send(AppEvent::DockerConsoleOutput(line)).is_err() {
                    break;
                }
            }
        });
    }

    if let Some(stderr) = stderr {
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let line = keep_ansi_line(&line);
                if tx_clone.send(AppEvent::DockerConsoleOutput(line)).is_err() {
                    break;
                }
            }
        });
    }

    // Wait for attach to finish and then notify the app.
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        let _ = child.wait().await;
        if let Some(lock) = CONSOLE_STDIN.get() {
            let mut guard = lock.lock().await;
            *guard = None;
        }
        let _ = tx_clone.send(AppEvent::DockerConsoleClosed);
    });

    Ok(())
}

#[cfg(not(unix))]
async fn send_console_line_pipes(line: String) -> Result<()> {
    if let Some(lock) = CONSOLE_STDIN.get() {
        let mut guard = lock.lock().await;
        if let Some(stdin) = guard.as_mut() {
            stdin.write_all(line.as_bytes()).await?;
            stdin.write_all(b"\n").await?;
        }
    }
    Ok(())
}

#[cfg(not(unix))]
async fn detach_console_pipes() -> Result<()> {
    if let Some(lock) = CONSOLE_STDIN.get() {
        let mut guard = lock.lock().await;
        if let Some(stdin) = guard.as_mut() {
            // Ctrl-P, Ctrl-Q
            stdin.write_all(&[0x10, 0x11]).await?;
        }
    }
    Ok(())
}

// In-app console implementations ------------------------------------------------

/// Open an in-app console for the given container:
///
/// - Fetches recent logs via `docker logs --tail` and sends them to the app.
/// - Starts an interactive session (PTY on unix, plain pipes otherwise).
/// - Streams output lines back as `DockerConsoleOutput` events.
#[cfg(not(unix))]
pub async fn open_console(tx: &UnboundedSender<AppEvent>, id: &str) -> Result<()> {
    open_console_pipes(tx, id).await
}

/// Send a single line of input into the active Docker console, if any.
#[cfg(not(unix))]
pub async fn send_console_line(line: String) -> Result<()> {
    send_console_line_pipes(line).await
}

/// Detach from the active Docker console (without stopping the container).
#[cfg(not(unix))]
pub async fn detach_console() -> Result<()> {
    detach_console_pipes().await
}

// ---------------- PTY-based console (unix: better TTY emulation) ---------------

#[cfg(unix)]
use portable_pty::{native_pty_system, CommandBuilder, PtySize};

#[cfg(unix)]
use std::io::{BufRead as StdBufRead, BufReader as StdBufReader, Write as StdWrite};

#[cfg(unix)]
use std::sync::OnceLock;

#[cfg(unix)]
use tokio::sync::Mutex;

/// Writer handle into the PTY for the current console session (if any).
#[cfg(unix)]
static CONSOLE_PTY_WRITER: OnceLock<Mutex<Option<Box<dyn StdWrite + Send>>>> = OnceLock::new();

#[cfg(unix)]
fn console_pty_writer() -> &'static Mutex<Option<Box<dyn StdWrite + Send>>> {
    CONSOLE_PTY_WRITER.get_or_init(|| Mutex::new(None))
}

/// Open console using a real PTY (unix only).
#[cfg(unix)]
pub async fn open_console(tx: &UnboundedSender<AppEvent>, id: &str) -> Result<()> {
    // 1) Fetch recent logs (same as non-unix path).
    let logs_output = Command::new("docker")
        .args(["logs", "--tail", "200", id])
        .output()
        .await?;

    if logs_output.status.success() {
        let stdout = String::from_utf8_lossy(&logs_output.stdout);
        let mut lines: Vec<String> = stdout.lines().map(|s| keep_ansi_line(s)).collect();
        const MAX_LINES: usize = 2000;
        if lines.len() > MAX_LINES {
            let excess = lines.len() - MAX_LINES;
            lines.drain(0..excess);
        }
        let _ = tx.send(AppEvent::DockerConsoleReset(lines));
    } else {
        let _ = tx.send(AppEvent::DockerConsoleReset(vec![format!(
            "(failed to fetch logs: {})",
            logs_output.status
        )]));
    }

    // 2) Launch `docker attach` inside a PTY for real TTY behavior.
    let pty_system = native_pty_system();
    let pair = pty_system.openpty(PtySize {
        rows: 40,
        cols: 120,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    let mut cmd = CommandBuilder::new("docker");
    cmd.arg("attach");
    cmd.arg("--sig-proxy=false");
    cmd.arg(id.to_string());

    let mut child = pair.slave.spawn_command(cmd)?;

    // Clone a reader and take the writer from the master PTY.
    let reader = pair.master.try_clone_reader()?;
    let writer = pair.master.take_writer()?;

    *console_pty_writer().lock().await = Some(writer);

    // Stream PTY output back into the TUI on a blocking thread.
    let tx_clone = tx.clone();
    std::thread::spawn(move || {
        let mut reader = StdBufReader::new(reader);
        let mut buf = String::new();
        loop {
            buf.clear();
            match reader.read_line(&mut buf) {
                Ok(0) => break,
                Ok(_) => {
                    let line = keep_ansi_line(&buf);
                    if tx_clone.send(AppEvent::DockerConsoleOutput(line)).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        let _ = child.wait();
        if let Some(lock) = CONSOLE_PTY_WRITER.get() {
            let mut guard = lock.blocking_lock();
            *guard = None;
        }
        let _ = tx_clone.send(AppEvent::DockerConsoleClosed);
    });

    Ok(())
}

/// Send a single line of input into the active PTY-based console, if any.
#[cfg(unix)]
pub async fn send_console_line(line: String) -> Result<()> {
    if let Some(lock) = CONSOLE_PTY_WRITER.get() {
        let mut guard = lock.lock().await;
        if let Some(writer) = guard.as_mut() {
            writer.write_all(line.as_bytes())?;
            writer.write_all(b"\n")?;
            writer.flush()?;
        }
    }
    Ok(())
}

/// Detach from the active PTY-based console (without stopping the container).
#[cfg(unix)]
pub async fn detach_console() -> Result<()> {
    if let Some(lock) = CONSOLE_PTY_WRITER.get() {
        let mut guard = lock.lock().await;
        if let Some(writer) = guard.as_mut() {
            // Docker default detach sequence: Ctrl-P, Ctrl-Q.
            writer.write_all(&[0x10, 0x11])?;
            writer.flush()?;
        }
    }
    Ok(())
}


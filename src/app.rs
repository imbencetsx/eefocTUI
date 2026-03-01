use crate::config::Config;
use crate::console;
use crate::events::AppEvent;
use crate::models::{container::Container, metrics::SystemMetrics};
use crate::services;
use crate::ui::views::View;
use ratatui::widgets::ListState;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NetworkSubView {
    #[default]
    Manager,
    PortChecker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PortInputFocus {
    #[default]
    Host,
    Ports,
}

#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub ip: String,
    pub mac: String,
    pub is_up: bool,
}

#[derive(Debug, Clone)]
pub struct NetworkConnection {
    pub protocol: String,
    pub local_addr: String,
    pub remote_addr: String,
    pub state: String,
    pub pid: u32,
}

#[derive(Debug, Clone)]
pub struct PortScanResult {
    pub port: u16,
    pub service: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PortPreset {
    #[default]
    Custom,
    Common,
    Web,
    Database,
    All,
}

impl PortPreset {
    pub fn ports(&self) -> Vec<u16> {
        match self {
            PortPreset::Custom => vec![],
            PortPreset::Common => vec![21, 22, 23, 25, 53, 80, 110, 143, 443, 993, 995, 3306, 3389, 5432, 8080, 8443],
            PortPreset::Web => vec![80, 443, 8080, 8443, 3000, 4000, 5000, 8000, 9000],
            PortPreset::Database => vec![3306, 5432, 27017, 6379, 11211, 9200, 9300],
            PortPreset::All => (1..=1024).collect(),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            PortPreset::Custom => "Custom",
            PortPreset::Common => "Common",
            PortPreset::Web => "Web",
            PortPreset::Database => "Database",
            PortPreset::All => "All (1-1024)",
        }
    }
}

#[derive(Debug, Default)]
pub struct NetworkState {
    pub sub_view: NetworkSubView,
    pub interfaces: Vec<NetworkInterface>,
    pub connections: Vec<NetworkConnection>,
    pub selected_interface: usize,
    pub selected_connection: usize,
    pub port_input_host: String,
    pub port_input_host_cursor: usize,
    pub port_input_ports: String,
    pub port_input_ports_cursor: usize,
    pub port_preset: PortPreset,
    pub input_focus: PortInputFocus,
    pub open_ports: Vec<PortScanResult>,
    pub closed_ports: Vec<u16>,
    pub scanning_status: Option<String>,
}

#[derive(Debug)]
pub struct DockerState {
    pub containers: Vec<Container>,
    pub selected: usize,
    pub table_scroll: usize,
    pub loading: bool,
    pub error: Option<String>,
    pub confirm_delete: bool,
    pub delete_input: String,
    // In-app console state
    pub console_open: bool,
    pub console_container_id: Option<String>,
    pub console_title: Option<String>,
    pub console_lines: Vec<String>,
    pub console_input: String,
    pub console_cursor: usize,
    pub console_scroll_offset: usize, // Scroll position (0 = bottom, higher = scrolled up)
    pub console_list_state: ListState,
}

impl Default for DockerState {
    fn default() -> Self {
        Self {
            containers: Vec::new(),
            selected: 0,
            table_scroll: 0,
            loading: false,
            error: None,
            confirm_delete: false,
            delete_input: String::new(),
            console_open: false,
            console_container_id: None,
            console_title: None,
            console_lines: Vec::new(),
            console_input: String::new(),
            console_cursor: 0,
            console_scroll_offset: 0,
            console_list_state: ListState::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct SystemState {
    pub metrics: Option<SystemMetrics>,
    pub last_updated_tick: u64,
    pub selected_disk: usize,
}

#[derive(Debug)]
pub struct App {
    pub config: Config,
    pub current_view: View,
    pub docker: DockerState,
    pub system: SystemState,
    pub network: NetworkState,
    pub should_quit: bool,
    pub tick_count: u64,
    pub tx: UnboundedSender<AppEvent>,
}

impl App {
    pub fn new(config: Config, tx: UnboundedSender<AppEvent>) -> Self {
        Self {
            config,
            current_view: View::Docker,
            docker: DockerState::default(),
            system: SystemState::default(),
            network: NetworkState::default(),
            should_quit: false,
            tick_count: 0,
            tx,
        }
    }

    pub fn next_view(&mut self) {
        self.current_view = self.current_view.next();
    }

    pub fn prev_view(&mut self) {
        self.current_view = self.current_view.prev();
    }

    pub fn on_tick(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
    }

    pub fn update(&mut self, event: AppEvent) {
        const MAX_CONSOLE_LINES: usize = 2000;

        match event {
            AppEvent::Quit => self.should_quit = true,
            AppEvent::NextView => self.next_view(),
            AppEvent::PrevView => self.prev_view(),
            AppEvent::Tick => self.on_tick(),
            AppEvent::DockerContainersUpdated(containers) => {
                self.docker.loading = false;
                self.docker.containers = containers;
                if self.docker.selected >= self.docker.containers.len() {
                    self.docker.selected = self
                        .docker
                        .containers
                        .is_empty()
                        .then_some(0)
                        .unwrap_or(0);
                }
            }
            AppEvent::DockerError(err) => {
                self.docker.loading = false;
                self.docker.error = Some(err);
            }
            AppEvent::MoveSelectionUp => match self.current_view {
                View::Docker => {
                    if !self.docker.containers.is_empty() {
                        if self.docker.selected == 0 {
                            self.docker.selected = self.docker.containers.len() - 1;
                        } else {
                            self.docker.selected -= 1;
                        }
                        if self.docker.selected < self.docker.table_scroll {
                            self.docker.table_scroll = self.docker.selected;
                        }
                    }
                }
                View::System => {
                    let len = self
                        .system
                        .metrics
                        .as_ref()
                        .map(|m| m.disks.len())
                        .unwrap_or(0);
                    if len > 0 {
                        if self.system.selected_disk == 0 {
                            self.system.selected_disk = len - 1;
                        } else {
                            self.system.selected_disk -= 1;
                        }
                    }
                }
                View::Network => {
                    match self.network.sub_view {
                        NetworkSubView::Manager => {
                            if self.network.selected_interface > 0 {
                                self.network.selected_interface -= 1;
                            }
                        }
                        NetworkSubView::PortChecker => {}
                    }
                }
                View::Help => {}
            },
            AppEvent::MoveSelectionDown => match self.current_view {
                View::Docker => {
                    if !self.docker.containers.is_empty() {
                        self.docker.selected =
                            (self.docker.selected + 1) % self.docker.containers.len();
                        let max_visible = 10;
                        if self.docker.selected >= self.docker.table_scroll + max_visible {
                            self.docker.table_scroll = self.docker.selected - max_visible + 1;
                        }
                    }
                }
                View::System => {
                    let len = self
                        .system
                        .metrics
                        .as_ref()
                        .map(|m| m.disks.len())
                        .unwrap_or(0);
                    if len > 0 {
                        self.system.selected_disk = (self.system.selected_disk + 1) % len;
                    }
                }
                View::Network => {
                    match self.network.sub_view {
                        NetworkSubView::Manager => {
                            let len = self.network.interfaces.len();
                            if len > 0 {
                                self.network.selected_interface = (self.network.selected_interface + 1) % len;
                            }
                        }
                        NetworkSubView::PortChecker => {}
                    }
                }
                View::Help => {}
            },
            AppEvent::DockerStartSelected => {
                if self.current_view == View::Docker {
                    if let Some(container) =
                        self.docker.containers.get(self.docker.selected).cloned()
                    {
                        let tx = self.tx.clone();
                        tokio::spawn(async move {
                            if let Err(err) =
                                services::docker::start_container(&container.id).await
                            {
                                let _ = tx.send(AppEvent::DockerError(format!(
                                    "Failed to start {}: {err}",
                                    container.name
                                )));
                            } else if let Err(err) =
                                services::docker::refresh_containers(&tx).await
                            {
                                let _ = tx.send(AppEvent::DockerError(format!(
                                    "Failed to refresh containers: {err}"
                                )));
                            }
                        });
                    }
                }
            }
            AppEvent::DockerStopSelected => {
                if self.current_view == View::Docker {
                    if let Some(container) =
                        self.docker.containers.get(self.docker.selected).cloned()
                    {
                        let tx = self.tx.clone();
                        tokio::spawn(async move {
                            if let Err(err) =
                                services::docker::stop_container(&container.id).await
                            {
                                let _ = tx.send(AppEvent::DockerError(format!(
                                    "Failed to stop {}: {err}",
                                    container.name
                                )));
                            } else if let Err(err) =
                                services::docker::refresh_containers(&tx).await
                            {
                                let _ = tx.send(AppEvent::DockerError(format!(
                                    "Failed to refresh containers: {err}"
                                )));
                            }
                        });
                    }
                }
            }
            AppEvent::DockerRestartSelected => {
                if self.current_view == View::Docker {
                    if let Some(container) =
                        self.docker.containers.get(self.docker.selected).cloned()
                    {
                        let tx = self.tx.clone();
                        tokio::spawn(async move {
                            if let Err(err) =
                                services::docker::restart_container(&container.id).await
                            {
                                let _ = tx.send(AppEvent::DockerError(format!(
                                    "Failed to restart {}: {err}",
                                    container.name
                                )));
                            } else if let Err(err) =
                                services::docker::refresh_containers(&tx).await
                            {
                                let _ = tx.send(AppEvent::DockerError(format!(
                                    "Failed to refresh containers: {err}"
                                )));
                            }
                        });
                    }
                }
            }
            AppEvent::DockerDeleteKey => {
                if self.current_view == View::Docker && !self.docker.console_open {
                    self.docker.confirm_delete = true;
                    self.docker.delete_input.clear();
                    console::set_console_active(true);
                }
            }
            AppEvent::DockerTableScrollUp => {
                if self.current_view == View::Docker {
                    if self.docker.table_scroll > 0 {
                        self.docker.table_scroll -= 1;
                    }
                }
            }
            AppEvent::DockerTableScrollDown => {
                if self.current_view == View::Docker {
                    let max_scroll = self.docker.containers.len().saturating_sub(1);
                    if self.docker.table_scroll < max_scroll {
                        self.docker.table_scroll += 1;
                    }
                }
            }
            AppEvent::SystemMetricsUpdated(metrics) => {
                self.system.metrics = Some(metrics);
                self.system.last_updated_tick = self.tick_count;
                // Keep selection in range.
                if let Some(m) = &self.system.metrics {
                    if self.system.selected_disk >= m.disks.len() {
                        self.system.selected_disk = 0;
                    }
                }
            }
            AppEvent::ClearError => {
                self.docker.error = None;
                if self.docker.confirm_delete {
                    self.docker.confirm_delete = false;
                    self.docker.delete_input.clear();
                    console::set_console_active(false);
                }
            }
            AppEvent::DockerAttachSelected => {
                if self.current_view == View::Docker {
                    if let Some(container) =
                        self.docker.containers.get(self.docker.selected).cloned()
                    {
                        self.docker.console_open = true;
                        self.docker.console_container_id = Some(container.id.clone());
                        self.docker.console_title = Some(container.name.clone());
                        self.docker.console_lines.clear();
                        self.docker.console_input.clear();
                        self.docker.console_cursor = 0;
                        self.docker.console_scroll_offset = 0;
                        console::set_console_active(true);

                        let tx = self.tx.clone();
                        tokio::spawn(async move {
                            if let Err(err) =
                                services::docker::open_console(&tx, &container.id).await
                            {
                                let _ = tx.send(AppEvent::DockerError(format!(
                                    "Failed to open console for {}: {err}",
                                    container.name
                                )));
                            }
                        });
                    }
                }
            }
            AppEvent::DockerConsoleReset(lines) => {
                self.docker.console_lines = lines;
                if self.docker.console_lines.len() > MAX_CONSOLE_LINES {
                    let excess = self.docker.console_lines.len() - MAX_CONSOLE_LINES;
                    self.docker.console_lines.drain(0..excess);
                }
                self.docker.console_open = true;
                self.docker.console_cursor = 0;
                self.docker.console_scroll_offset = 0; // Auto-scroll to bottom
                console::set_console_active(true);
            }
            AppEvent::DockerConsoleOutput(line) => {
                let was_at_bottom = self.docker.console_scroll_offset == 0;
                self.docker.console_lines.push(line);
                if self.docker.console_lines.len() > MAX_CONSOLE_LINES {
                    let excess = self.docker.console_lines.len() - MAX_CONSOLE_LINES;
                    self.docker.console_lines.drain(0..excess);
                }
                // Auto-scroll to bottom if user was already at bottom
                if was_at_bottom {
                    self.docker.console_scroll_offset = 0;
                }
            }
            AppEvent::DockerConsoleScrollUp => {
                if self.docker.console_open {
                    // Scroll up: increase offset (0 = bottom, higher = scrolled up)
                    let max_offset = self.docker.console_lines.len().saturating_sub(1);
                    self.docker.console_scroll_offset = (self.docker.console_scroll_offset + 10)
                        .min(max_offset);
                }
            }
            AppEvent::DockerConsoleScrollDown => {
                if self.docker.console_open {
                    // Scroll down: decrease offset (toward bottom)
                    self.docker.console_scroll_offset = self
                        .docker
                        .console_scroll_offset
                        .saturating_sub(10);
                }
            }
            AppEvent::DockerConsoleScrollToBottom => {
                if self.docker.console_open {
                    self.docker.console_scroll_offset = 0;
                }
            }
            AppEvent::DockerConsoleClosed => {
                self.docker.console_open = false;
                self.docker.console_container_id = None;
                self.docker.console_title = None;
                self.docker.console_input.clear();
                console::set_console_active(false);
            }
            AppEvent::DockerConsoleKey(ch) => {
                if !ch.is_control() {
                    if self.docker.confirm_delete {
                        self.docker.delete_input.push(ch);
                    } else if self.docker.console_open {
                        self.docker.console_input.insert(self.docker.console_cursor, ch);
                        self.docker.console_cursor += 1;
                    }
                }
            }
            AppEvent::DockerConsoleBackspace => {
                if self.docker.confirm_delete {
                    self.docker.delete_input.pop();
                } else if self.docker.console_open && self.docker.console_cursor > 0 {
                    self.docker.console_cursor -= 1;
                    self.docker.console_input.remove(self.docker.console_cursor);
                }
            }
            AppEvent::DockerConsoleCursorLeft => {
                if self.docker.console_open && self.docker.console_cursor > 0 {
                    self.docker.console_cursor -= 1;
                }
            }
            AppEvent::DockerConsoleCursorRight => {
                if self.docker.console_open && self.docker.console_cursor < self.docker.console_input.len() {
                    self.docker.console_cursor += 1;
                }
            }
            AppEvent::DockerConsoleEnter => {
                if self.docker.confirm_delete {
                    if let Some(container) = self.docker.containers.get(self.docker.selected).cloned() {
                        if self.docker.delete_input.trim() == container.name {
                            self.docker.confirm_delete = false;
                            self.docker.delete_input.clear();
                            console::set_console_active(false);

                            let tx = self.tx.clone();
                            tokio::spawn(async move {
                                if let Err(err) = services::docker::delete_container(&container.id).await {
                                    let _ = tx.send(AppEvent::DockerError(format!(
                                        "Failed to delete {}: {err}",
                                        container.name
                                    )));
                                } else if let Err(err) = services::docker::refresh_containers(&tx).await {
                                    let _ = tx.send(AppEvent::DockerError(format!(
                                        "Failed to refresh containers: {err}"
                                    )));
                                }
                            });
                        }
                    }
                } else if self.docker.console_open && !self.docker.console_input.is_empty() {
                    let line = self.docker.console_input.clone();
                    self.docker.console_input.clear();
                    self.docker.console_cursor = 0;
                    let tx = self.tx.clone();
                    tokio::spawn(async move {
                        if let Err(err) =
                            services::docker::send_console_line(line.clone()).await
                        {
                            let _ = tx.send(AppEvent::DockerError(format!(
                                "Failed to send console input: {err}"
                            )));
                        }
                    });
                }
            }
            AppEvent::DockerConsoleDetach => {
                if self.docker.confirm_delete {
                    self.docker.confirm_delete = false;
                    self.docker.delete_input.clear();
                    console::set_console_active(false);
                } else if self.docker.console_open {
                    self.docker.console_open = false;
                    self.docker.console_container_id = None;
                    self.docker.console_title = None;
                    self.docker.console_input.clear();
                    self.docker.console_scroll_offset = 0;
                    console::set_console_active(false);

                    let tx = self.tx.clone();
                    tokio::spawn(async move {
                        if let Err(err) = services::docker::detach_console().await {
                            let _ = tx.send(AppEvent::DockerError(format!(
                                "Failed to detach console: {err}"
                            )));
                        }
                    });
                }
            }
            // Network events
            AppEvent::NetworkNextSubView => {
                if self.current_view == View::Network {
                    self.network.sub_view = match self.network.sub_view {
                        NetworkSubView::Manager => NetworkSubView::PortChecker,
                        NetworkSubView::PortChecker => NetworkSubView::Manager,
                    };
                }
            }
            AppEvent::NetworkPrevSubView => {
                if self.current_view == View::Network {
                    self.network.sub_view = match self.network.sub_view {
                        NetworkSubView::Manager => NetworkSubView::PortChecker,
                        NetworkSubView::PortChecker => NetworkSubView::Manager,
                    };
                }
            }
            AppEvent::NetworkInputEnter => {
                if self.current_view == View::Network {
                    self.network.sub_view = NetworkSubView::PortChecker;
                }
            }
            AppEvent::NetworkInputExit => {}
            AppEvent::NetworkCyclePreset => {
                if self.current_view == View::Network && self.network.sub_view == NetworkSubView::PortChecker {
                    self.network.port_preset = match self.network.port_preset {
                        PortPreset::Custom => PortPreset::Common,
                        PortPreset::Common => PortPreset::Web,
                        PortPreset::Web => PortPreset::Database,
                        PortPreset::Database => PortPreset::All,
                        PortPreset::All => PortPreset::Custom,
                    };
                    if self.network.port_preset != PortPreset::Custom {
                        let ports: Vec<String> = self.network.port_preset.ports()
                            .iter()
                            .map(|p| p.to_string())
                            .collect();
                        self.network.port_input_ports = ports.join(",");
                        self.network.port_input_ports_cursor = self.network.port_input_ports.len();
                    }
                }
            }
            AppEvent::NetworkUpdated(interfaces, connections) => {
                self.network.interfaces = interfaces;
                self.network.connections = connections;
                if self.network.selected_interface >= self.network.interfaces.len() {
                    self.network.selected_interface = 0;
                }
                if self.network.selected_connection >= self.network.connections.len() {
                    self.network.selected_connection = 0;
                }
            }
            AppEvent::NetworkPortInputKey(ch) => {
                if self.current_view == View::Network && self.network.sub_view == NetworkSubView::PortChecker {
                    match self.network.input_focus {
                        PortInputFocus::Host => {
                            self.network.port_input_host.insert(self.network.port_input_host_cursor, ch);
                            self.network.port_input_host_cursor += 1;
                        }
                        PortInputFocus::Ports => {
                            self.network.port_input_ports.insert(self.network.port_input_ports_cursor, ch);
                            self.network.port_input_ports_cursor += 1;
                        }
                    }
                }
            }
            AppEvent::NetworkPortInputBackspace => {
                if self.current_view == View::Network && self.network.sub_view == NetworkSubView::PortChecker {
                    match self.network.input_focus {
                        PortInputFocus::Host => {
                            if self.network.port_input_host_cursor > 0 {
                                self.network.port_input_host_cursor -= 1;
                                self.network.port_input_host.remove(self.network.port_input_host_cursor);
                            }
                        }
                        PortInputFocus::Ports => {
                            if self.network.port_input_ports_cursor > 0 {
                                self.network.port_input_ports_cursor -= 1;
                                self.network.port_input_ports.remove(self.network.port_input_ports_cursor);
                            }
                        }
                    }
                }
            }
            AppEvent::NetworkPortInputCursorLeft => {
                if self.current_view == View::Network && self.network.sub_view == NetworkSubView::PortChecker {
                    match self.network.input_focus {
                        PortInputFocus::Host => {
                            if self.network.port_input_host_cursor > 0 {
                                self.network.port_input_host_cursor -= 1;
                            }
                        }
                        PortInputFocus::Ports => {
                            if self.network.port_input_ports_cursor > 0 {
                                self.network.port_input_ports_cursor -= 1;
                            }
                        }
                    }
                }
            }
            AppEvent::NetworkPortInputCursorRight => {
                if self.current_view == View::Network && self.network.sub_view == NetworkSubView::PortChecker {
                    match self.network.input_focus {
                        PortInputFocus::Host => {
                            if self.network.port_input_host_cursor < self.network.port_input_host.len() {
                                self.network.port_input_host_cursor += 1;
                            }
                        }
                        PortInputFocus::Ports => {
                            if self.network.port_input_ports_cursor < self.network.port_input_ports.len() {
                                self.network.port_input_ports_cursor += 1;
                            }
                        }
                    }
                }
            }
            AppEvent::NetworkPortInputTab => {
                if self.current_view == View::Network && self.network.sub_view == NetworkSubView::PortChecker {
                    self.network.input_focus = match self.network.input_focus {
                        PortInputFocus::Host => PortInputFocus::Ports,
                        PortInputFocus::Ports => PortInputFocus::Host,
                    };
                }
            }
            AppEvent::NetworkPortScanStart => {
                if self.current_view == View::Network && self.network.sub_view == NetworkSubView::PortChecker {
                    self.network.open_ports.clear();
                    self.network.closed_ports.clear();
                    self.network.scanning_status = Some("Scanning...".to_string());
                    
                    let host = self.network.port_input_host.clone();
                    let ports_str = self.network.port_input_ports.clone();
                    let tx = self.tx.clone();
                    
                    tokio::spawn(async move {
                        use std::net::{TcpStream, ToSocketAddrs};
                        use std::time::Duration;
                        
                        let ports: Vec<u16> = ports_str
                            .split(',')
                            .filter_map(|s| s.trim().parse::<u16>().ok())
                            .collect();
                        
                        for port in ports {
                            let addr_str = format!("{}:{}", host, port);
                            let open = addr_str.to_socket_addrs()
                                .ok()
                                .and_then(|mut addrs| addrs.next())
                                .and_then(|addr| {
                                    TcpStream::connect_timeout(&addr, Duration::from_millis(500)).ok()
                                })
                                .is_some();
                            
                            let service = if open {
                                get_service_name(port).to_string()
                            } else {
                                String::new()
                            };
                            
                            let _ = tx.send(AppEvent::NetworkPortScanResult(port, open, service));
                        }
                        
                        let _ = tx.send(AppEvent::NetworkPortScanResult(0, false, String::new()));
                    });
                }
            }
            AppEvent::NetworkPortScanResult(port, open, service) => {
                if port == 0 {
                    self.network.scanning_status = None;
                } else if open {
                    self.network.open_ports.push(PortScanResult { port, service });
                } else {
                    self.network.closed_ports.push(port);
                }
            }
        }
    }
}

fn get_service_name(port: u16) -> &'static str {
    match port {
        20 => "FTP Data",
        21 => "FTP",
        22 => "SSH",
        23 => "Telnet",
        25 => "SMTP",
        53 => "DNS",
        67 => "DHCP",
        68 => "DHCP",
        69 => "TFTP",
        80 => "HTTP",
        110 => "POP3",
        119 => "NNTP",
        123 => "NTP",
        135 => "RPC",
        137 => "NetBIOS",
        138 => "NetBIOS",
        139 => "NetBIOS",
        143 => "IMAP",
        161 => "SNMP",
        162 => "SNMP Trap",
        389 => "LDAP",
        443 => "HTTPS",
        445 => "SMB",
        465 => "SMTPS",
        514 => "Syslog",
        587 => "SMTP",
        636 => "LDAPS",
        993 => "IMAPS",
        995 => "POP3S",
        1433 => "MSSQL",
        1521 => "Oracle",
        3306 => "MySQL",
        3389 => "RDP",
        5432 => "PostgreSQL",
        5900 => "VNC",
        6379 => "Redis",
        8080 => "HTTP Alt",
        8443 => "HTTPS Alt",
        9200 => "Elasticsearch",
        9300 => "Elasticsearch",
        27017 => "MongoDB",
        _ => "Unknown",
    }
}


use crate::app::{App, NetworkSubView};
use crate::ui::icons::Icons;
use crate::ui::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, Paragraph, Row, Table};
use ratatui::Frame;

pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)].as_ref())
        .split(area);

    let sub_tabs = ["Network Manager", "Port Checker"];
    let active_sub = match app.network.sub_view {
        NetworkSubView::Manager => 0,
        NetworkSubView::PortChecker => 1,
    };

    let tab_spans: Vec<Span> = sub_tabs
        .iter()
        .enumerate()
        .flat_map(|(i, tab)| {
            let style = if i == active_sub {
                theme.accent_style()
            } else {
                theme.secondary_style()
            };
            let divider = if i < sub_tabs.len() - 1 {
                vec![Span::styled(" | ", theme.secondary_style())]
            } else {
                vec![]
            };
            vec![Span::styled(format!(" {} ", tab), style)]
                .into_iter()
                .chain(divider)
        })
        .collect();

    let title = Paragraph::new(Line::from(tab_spans))
        .block(theme.soft_block())
        .style(theme.base());
    frame.render_widget(title, chunks[0]);

    match app.network.sub_view {
        NetworkSubView::Manager => render_manager(frame, chunks[1], theme, app),
        NetworkSubView::PortChecker => render_port_checker(frame, chunks[1], theme, app),
    }
}

fn render_manager(frame: &mut Frame, area: Rect, theme: &Theme, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    let interfaces = &app.network.interfaces;
    let headers = ["Interface", "IP Address", "MAC Address", "Status"];

    let rows: Vec<Row> = interfaces
        .iter()
        .enumerate()
        .map(|(idx, iface)| {
            let is_selected = idx == app.network.selected_interface;
            let style = if is_selected {
                theme.accent_style().add_modifier(Modifier::BOLD)
            } else {
                theme.base()
            };
            Row::new(vec![
                Cell::from(iface.name.as_str()),
                Cell::from(iface.ip.as_str()),
                Cell::from(iface.mac.as_str()),
                Cell::from(if iface.is_up { "UP" } else { "DOWN" }),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(rows, [Constraint::Percentage(25); 4])
        .header(
            Row::new(headers.map(|h| Cell::from(h)))
                .style(
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                )
                .bottom_margin(1),
        )
        .block(theme.soft_block().title(Span::styled(
            format!(" {} Network Interfaces ", Icons::NETWORK),
            theme.secondary_style(),
        )))
        .column_spacing(2);

    frame.render_widget(table, chunks[0]);

    let connections = &app.network.connections;
    let conn_headers = [
        "Protocol",
        "Local Address",
        "Remote Address",
        "State",
        "PID",
    ];

    let conn_rows: Vec<Row> = connections
        .iter()
        .enumerate()
        .map(|(idx, conn)| {
            let is_selected = idx == app.network.selected_connection;
            let style = if is_selected {
                theme.accent_style().add_modifier(Modifier::BOLD)
            } else {
                theme.base()
            };
            Row::new(vec![
                Cell::from(conn.protocol.as_str()),
                Cell::from(conn.local_addr.as_str()),
                Cell::from(conn.remote_addr.as_str()),
                Cell::from(conn.state.as_str()),
                Cell::from(conn.pid.to_string()),
            ])
            .style(style)
        })
        .collect();

    let conn_table = Table::new(
        conn_rows,
        [
            Constraint::Length(10),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(20),
            Constraint::Length(8),
        ],
    )
    .header(
        Row::new(conn_headers.map(|h| Cell::from(h)))
            .style(
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )
            .bottom_margin(1),
    )
    .block(theme.soft_block().title(Span::styled(
        " Active Connections ",
        theme.secondary_style(),
    )))
    .column_spacing(2);

    frame.render_widget(conn_table, chunks[1]);
}

fn render_port_checker(frame: &mut Frame, area: Rect, theme: &Theme, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Percentage(40),
            Constraint::Percentage(40),
        ])
        .split(area);

    let host = &app.network.port_input_host;
    let host_cursor = app.network.port_input_host_cursor;
    let host_focused = app.network.input_focus == crate::app::PortInputFocus::Host;

    let host_line = Line::from(vec![
        Span::styled("Host: ", theme.accent_style()),
        Span::raw(&host[..host_cursor]),
        Span::styled(
            if host_cursor < host.len() {
                &host[host_cursor..host_cursor + 1]
            } else if host_focused {
                " "
            } else {
                ""
            },
            Style::default().bg(if host_focused {
                theme.accent
            } else {
                Color::Reset
            }),
        ),
        Span::raw(&host[host_cursor..]),
    ]);

    let ports = &app.network.port_input_ports;
    let ports_cursor = app.network.port_input_ports_cursor;
    let ports_focused = app.network.input_focus == crate::app::PortInputFocus::Ports;

    let ports_line = Line::from(vec![
        Span::styled("Ports: ", theme.accent_style()),
        Span::raw(&ports[..ports_cursor]),
        Span::styled(
            if ports_cursor < ports.len() {
                &ports[ports_cursor..ports_cursor + 1]
            } else if ports_focused {
                " "
            } else {
                ""
            },
            Style::default().bg(if ports_focused {
                theme.accent
            } else {
                Color::Reset
            }),
        ),
        Span::raw(&ports[ports_cursor..]),
    ]);

    let help_line = Line::from(vec![
        Span::styled("Tab", theme.secondary_style()),
        Span::raw(" switch field  "),
        Span::styled("< / >", theme.secondary_style()),
        Span::raw(" move cursor  "),
        Span::styled("p", theme.secondary_style()),
        Span::raw(" preset  "),
        Span::styled("Enter", theme.secondary_style()),
        Span::raw(" scan"),
    ]);

    let preset_line = Line::from(vec![
        Span::styled("Preset: ", theme.accent_style()),
        Span::styled(app.network.port_preset.label(), theme.matcha_style()),
    ]);

    let input_widget = Paragraph::new(vec![
        host_line,
        ports_line,
        preset_line,
        Line::from(""),
        help_line,
    ])
    .block(theme.soft_block().title(Span::styled(
        format!(" {} Port Scanner ", Icons::NETWORK),
        theme.secondary_style(),
    )))
    .style(theme.base());
    frame.render_widget(input_widget, chunks[0]);

    let open_headers = ["Port", "Service", "Status"];
    let open_rows: Vec<Row> = app
        .network
        .open_ports
        .iter()
        .map(|port| {
            Row::new(vec![
                Cell::from(port.port.to_string()),
                Cell::from(port.service.as_str()),
                Cell::from(Span::styled("OPEN", Style::default().fg(Color::Green))),
            ])
        })
        .collect();

    let open_table = Table::new(
        open_rows,
        [
            Constraint::Length(10),
            Constraint::Percentage(50),
            Constraint::Length(10),
        ],
    )
    .header(
        Row::new(open_headers.map(|h| Cell::from(h)))
            .style(
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )
            .bottom_margin(1),
    )
    .block(
        theme
            .soft_block()
            .title(Span::styled(" Open Ports ", theme.secondary_style())),
    )
    .column_spacing(2);

    frame.render_widget(open_table, chunks[1]);

    if let Some(ref scanning) = app.network.scanning_status {
        let status = Paragraph::new(Line::from(Span::styled(
            format!(" {} ", scanning),
            theme.matcha_style(),
        )))
        .block(theme.soft_block().title(" Status "))
        .style(theme.base());
        frame.render_widget(status, chunks[2]);
    } else {
        let closed_rows: Vec<Row> = app
            .network
            .closed_ports
            .iter()
            .map(|port| {
                Row::new(vec![
                    Cell::from(port.to_string()),
                    Cell::from(Span::styled("CLOSED", Style::default().fg(Color::Red))),
                ])
            })
            .collect();

        let closed_table = Table::new(
            closed_rows,
            [Constraint::Length(10), Constraint::Length(10)],
        )
        .header(
            Row::new(vec![Cell::from("Port"), Cell::from("Status")])
                .style(
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                )
                .bottom_margin(1),
        )
        .block(
            theme
                .soft_block()
                .title(Span::styled(" Closed Ports ", theme.secondary_style())),
        )
        .column_spacing(2);

        frame.render_widget(closed_table, chunks[2]);
    }
}

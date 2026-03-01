use crate::app::App;
use crate::ui::icons::Icons;
use crate::ui::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{BarChart, Block, Borders, Gauge, List, ListItem, Paragraph};
use ratatui::Frame;
use std::fs;
use std::path::Path;

pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Percentage(40),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ]
            .as_ref(),
        )
        .split(area);

    let title = Paragraph::new(Line::from(vec![Span::styled(
        format!(" {} System Metrics ", Icons::SYSTEM),
        theme.accent_style(),
    )]))
    .block(theme.soft_block())
    .style(theme.base());
    frame.render_widget(title, chunks[0]);

    if let Some(metrics) = &app.system.metrics {
        let cpu_color = match (app.tick_count / 10) % 3 {
            0 => theme.matcha,
            1 => theme.secondary,
            _ => theme.accent,
        };

        let cpu_data: Vec<(&str, u64)> = metrics
            .cpu_cores
            .iter()
            .map(|c| (c.name.as_str(), c.usage as u64))
            .collect();

        let cpu_chart = BarChart::default()
            .block(theme.soft_block().title(Span::styled(
                format!(" {} CPU usage ", Icons::CPU),
                theme.secondary_style(),
            )))
            .data(&cpu_data)
            .bar_width(4)
            .bar_gap(1)
            .bar_style(Style::default().fg(cpu_color).bg(cpu_color))
            .value_style(Style::default().fg(Color::Black).bg(cpu_color));

        frame.render_widget(cpu_chart, chunks[1]);

        let mem_used_gb = metrics.memory.used as f64 / (1024.0 * 1024.0 * 1024.0);
        let mem_total_gb = metrics.memory.total as f64 / (1024.0 * 1024.0 * 1024.0);
        let mem_ratio = if metrics.memory.total > 0 {
            metrics.memory.used as f64 / metrics.memory.total as f64
        } else {
            0.0
        };
        let mem_pct = mem_ratio * 100.0;

        let mem_gauge = Gauge::default()
            .block(theme.soft_block().title(Span::styled(
                format!(" {} Memory usage ", Icons::MEMORY),
                theme.secondary_style(),
            )))
            .gauge_style(
                Style::default()
                    .fg(theme.matcha)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            )
            .ratio(mem_ratio.clamp(0.0, 1.0))
            .label(format!(
                "{mem_pct:5.1}%  ({:.2} / {:.2} GiB)",
                mem_used_gb, mem_total_gb
            ));
        frame.render_widget(mem_gauge, chunks[2]);

        let disk_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)].as_ref())
            .split(chunks[3]);

        let disk_items: Vec<ListItem> = metrics
            .disks
            .iter()
            .enumerate()
            .map(|(idx, d)| {
                let used_gb = d.used as f64 / (1024.0 * 1024.0 * 1024.0);
                let total_gb = d.total as f64 / (1024.0 * 1024.0 * 1024.0);
                let pct = if d.total > 0 {
                    (d.used as f64 / d.total as f64) * 100.0
                } else {
                    0.0
                };
                let label = if d.name.to_lowercase() == "overlay" || d.name.is_empty() {
                    d.mount_point.clone()
                } else {
                    format!("{} ({})", d.name, d.mount_point)
                };

                let style = if idx == app.system.selected_disk {
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD)
                } else {
                    theme.base()
                };

                ListItem::new(format!(
                    "{label}\n  {pct:5.1}%  ({used:.2}/{total:.2} GiB)",
                    used = used_gb,
                    total = total_gb
                ))
                .style(style)
            })
            .collect();

        let disks_list = List::new(disk_items)
            .block(theme.soft_block().title(Span::styled(
                format!(" {} Disks ", Icons::DISK),
                theme.secondary_style(),
            )))
            .style(theme.base());
        frame.render_widget(disks_list, disk_chunks[0]);

        let selected_mount = metrics
            .disks
            .get(app.system.selected_disk)
            .map(|d| d.mount_point.as_str());
        let file_items: Vec<ListItem> = selected_mount
            .and_then(|mp| list_dir_preview(Path::new(mp)).ok())
            .unwrap_or_else(|| vec![ListItem::new("n/a")]);

        let files_list = List::new(file_items)
            .block(theme.soft_block().title(Span::styled(
                format!(" {} Files ", Icons::FILE),
                theme.secondary_style(),
            )))
            .style(theme.base());
        frame.render_widget(files_list, disk_chunks[1]);

        let net = &metrics.network;
        let net_text = Line::from(vec![
            Span::raw("Network total since boot - "),
            Span::styled(
                format!("v {} B  ", net.received),
                Style::default().fg(Color::Cyan),
            ),
            Span::styled(
                format!("^ {} B", net.transmitted),
                Style::default().fg(Color::Magenta),
            ),
        ]);
        let net_p = Paragraph::new(net_text)
            .block(theme.soft_block().title(Span::styled(
                format!(" {} Network summary ", Icons::NETWORK),
                theme.secondary_style(),
            )))
            .style(theme.base());
        frame.render_widget(net_p, chunks[4]);
    } else {
        let loading = Paragraph::new("Collecting system metrics...")
            .block(Block::default().borders(Borders::ALL))
            .style(theme.base());
        frame.render_widget(loading, chunks[1]);
    }
}

fn list_dir_preview(path: &Path) -> std::io::Result<Vec<ListItem<'static>>> {
    let mut entries: Vec<String> = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let file_name = entry.file_name().to_string_lossy().to_string();
        let meta = entry.metadata();
        let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
        let label = if is_dir {
            format!("{}/", file_name)
        } else {
            file_name
        };
        entries.push(label);
        if entries.len() >= 18 {
            break;
        }
    }

    entries.sort();
    Ok(entries.into_iter().map(|s| ListItem::new(s)).collect())
}

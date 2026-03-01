use crate::app::App;
use crate::ui::icons::Icons;
use crate::ui::theme::Theme;
use ansi_parser::{AnsiParser, AnsiSequence, Output};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table};
use ratatui::Frame;

fn ansi_to_ratatui_color(args: &[u8]) -> Option<Color> {
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            30 => return Some(Color::Black),
            31 => return Some(Color::Red),
            32 => return Some(Color::Green),
            33 => return Some(Color::Yellow),
            34 => return Some(Color::Blue),
            35 => return Some(Color::Magenta),
            36 => return Some(Color::Cyan),
            37 => return Some(Color::White),
            90 => return Some(Color::DarkGray),
            91 => return Some(Color::LightRed),
            92 => return Some(Color::LightGreen),
            93 => return Some(Color::LightYellow),
            94 => return Some(Color::LightBlue),
            95 => return Some(Color::LightMagenta),
            96 => return Some(Color::LightCyan),
            97 => return Some(Color::White),
            38 if i + 2 < args.len() && args[i + 1] == 5 => {
                let idx = args[i + 2];
                return Some(Color::Indexed(idx));
            }
            38 if i + 4 < args.len() && args[i + 1] == 2 => {
                return Some(Color::Rgb(args[i + 2], args[i + 3], args[i + 4]));
            }
            _ => {
                i += 1;
                continue;
            }
        }
    }
    None
}

fn parse_ansi_line(line: &str) -> Line<'_> {
    let mut spans: Vec<Span<'_>> = Vec::new();
    let mut current_fg: Option<Color> = None;
    let mut current_bold = false;

    for output in line.ansi_parse() {
        match output {
            Output::TextBlock(text) => {
                let style = if let Some(fg) = current_fg {
                    let mut s = Style::default().fg(fg);
                    if current_bold {
                        s = s.add_modifier(Modifier::BOLD);
                    }
                    s
                } else if current_bold {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                spans.push(Span::styled(text, style));
            }
            Output::Escape(seq) => match seq {
                AnsiSequence::SetGraphicsMode(args) => {
                    let mut i = 0;
                    while i < args.len() {
                        match args[i] {
                            0 => {
                                current_fg = None;
                                current_bold = false;
                            }
                            1 => current_bold = true,
                            22 => current_bold = false,
                            30..=37 | 90..=97 => {
                                current_fg = ansi_to_ratatui_color(&args);
                            }
                            38 => {
                                current_fg = ansi_to_ratatui_color(&args);
                                break;
                            }
                            39 => current_fg = None,
                            _ => {}
                        }
                        i += 1;
                    }
                }
                _ => {}
            },
        }
    }

    if spans.is_empty() {
        Line::from(line)
    } else {
        Line::from(spans)
    }
}

fn get_container_color(name: &str, image: &str) -> Option<Color> {
    let name_lower = name.to_lowercase();
    let image_lower = image.to_lowercase();

    if name_lower.contains("minecraft")
        || image_lower.contains("minecraft")
        || name_lower.contains("eefoc")
    {
        Some(Color::Rgb(85, 170, 85))
    } else if name_lower.contains("terminal")
        || image_lower.contains("terminal")
        || name_lower.contains("ttyd")
        || image_lower.contains("tty")
        || name_lower.contains("shell")
        || name_lower.contains("bash")
        || name_lower.contains("console")
        || name_lower.contains("alpine")
    {
        Some(Color::Rgb(100, 200, 220))
    } else {
        None
    }
}

pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, app: &mut App) {
    let has_console = app.docker.console_open;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(if has_console {
            &[
                Constraint::Length(3),
                Constraint::Percentage(40),
                Constraint::Percentage(50),
            ][..]
        } else {
            &[Constraint::Length(3), Constraint::Min(5)][..]
        })
        .split(area);

    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" {} Containers ", Icons::DOCKER),
            theme.accent_style(),
        ),
        Span::styled(
            " (Enter: console, Ins: detach, s: stop, r: restart, Del: delete)",
            theme.secondary_style(),
        ),
    ]))
    .block(theme.soft_block())
    .style(theme.base());
    frame.render_widget(title, chunks[0]);

    let headers = ["", "Name", "Image", "Status", "CPU", "Memory", "Uptime"];
    let header_style = Style::default()
        .fg(theme.accent)
        .add_modifier(Modifier::BOLD);

    let scroll = app.docker.table_scroll;
    let visible_rows: Vec<Row> = app
        .docker
        .containers
        .iter()
        .enumerate()
        .skip(scroll)
        .map(|(idx, c)| {
            let is_selected = idx == app.docker.selected;

            let _pulse_stage = (app.tick_count / 6) % 3;

            let special_color = get_container_color(&c.name, &c.image);

            let (status_icon, base_style) =
                if c.status.contains("Up") || c.status.contains("running") {
                    let style = if let Some(color) = special_color {
                        Style::default().fg(color)
                    } else {
                        theme.matcha_style()
                    };
                    (Icons::STATUS_RUNNING, style)
                } else if c.status.contains("Exited") || c.status.contains("stopped") {
                    (Icons::STATUS_STOPPED, Style::default().fg(theme.error))
                } else {
                    (Icons::STATUS_OTHER, theme.secondary_style())
                };

            let row_style = if is_selected {
                base_style.add_modifier(Modifier::BOLD).bg(if is_selected {
                    Color::Indexed(235)
                } else {
                    Color::Reset
                })
            } else {
                base_style
            };

            Row::new(vec![
                Cell::from(status_icon),
                Cell::from(c.name.clone()),
                Cell::from(c.image.clone()),
                Cell::from(c.status.clone()),
                Cell::from(format!("{}%", c.cpu_percent)),
                Cell::from(c.memory.clone()),
                Cell::from(c.uptime.clone()),
            ])
            .style(row_style)
        })
        .collect();

    let widths = [
        Constraint::Length(1),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Length(6),
        Constraint::Length(10),
        Constraint::Percentage(15),
    ];

    let table = Table::new(visible_rows, widths)
        .header(
            Row::new(headers.map(|h| Cell::from(h)))
                .style(header_style)
                .bottom_margin(1),
        )
        .block(
            theme
                .soft_block()
                .title(Span::styled(" Records ", theme.secondary_style())),
        )
        .column_spacing(1);

    frame.render_widget(table, chunks[1]);

    if has_console {
        let mut items: Vec<ListItem> = app
            .docker
            .console_lines
            .iter()
            .map(|line| ListItem::new(parse_ansi_line(line)))
            .collect();

        items.push(ListItem::new(Line::from(vec![
            Span::styled("> ", theme.matcha_style()),
            Span::raw(&app.docker.console_input[..app.docker.console_cursor]),
            Span::styled(
                if app.docker.console_cursor < app.docker.console_input.len() {
                    &app.docker.console_input
                        [app.docker.console_cursor..app.docker.console_cursor + 1]
                } else {
                    " "
                },
                Style::default().bg(theme.accent),
            ),
            Span::raw(&app.docker.console_input[app.docker.console_cursor..]),
        ])));

        let total_items = items.len();

        let scroll_offset = app.docker.console_scroll_offset;

        if total_items == 0 {
            app.docker.console_list_state.select(None);
        } else if scroll_offset == 0 {
            app.docker
                .console_list_state
                .select(Some(total_items.saturating_sub(1)));
        } else {
            let selected_idx = total_items.saturating_sub(scroll_offset + 1);
            app.docker
                .console_list_state
                .select(Some(selected_idx.min(total_items.saturating_sub(1))));
        }

        let console_title = app
            .docker
            .console_title
            .as_deref()
            .unwrap_or("selected container");

        let list = List::new(items)
            .block(theme.soft_block().title(Span::styled(
                format!(" {} Console ({}) ", Icons::CONSOLE, console_title),
                theme.secondary_style(),
            )))
            .style(theme.base());

        frame.render_stateful_widget(list, chunks[2], &mut app.docker.console_list_state);
    }

    if app.docker.confirm_delete {
        let popup_area = centered_rect(60, 30, area);
        let block = theme.soft_block().title(Span::styled(
            format!(" {} Confirm Delete ", Icons::DELETE),
            theme.error,
        ));
        let name = app
            .docker
            .containers
            .get(app.docker.selected)
            .map(|c| c.name.as_str())
            .unwrap_or("selected container");
        let text = Paragraph::new(vec![
            Line::from(Span::styled(
                format!("To delete '{name}', type its name exactly:"),
                Style::default()
                    .fg(theme.error)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!(" > {}_", app.docker.delete_input),
                theme.accent_style(),
            )),
            Line::from(""),
            Line::from(Span::styled(
                " Press Enter to confirm, Ins to cancel",
                theme.secondary_style(),
            )),
        ])
        .block(block)
        .style(theme.base());
        frame.render_widget(Clear, popup_area);
        frame.render_widget(text, popup_area);
    }

    if let Some(err) = &app.docker.error {
        let popup_area = centered_rect(70, 20, area);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Docker Error ")
            .border_style(Style::default().fg(theme.error));
        let text = Paragraph::new(err.as_str())
            .style(Style::default().fg(theme.error))
            .block(block);
        frame.render_widget(Clear, popup_area);
        frame.render_widget(text, popup_area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    let vertical = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1]);

    vertical[1]
}

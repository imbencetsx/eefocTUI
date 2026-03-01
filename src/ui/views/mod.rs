pub mod docker;
pub mod help;
pub mod network;
pub mod system;

use crate::app::App;
use crate::ui::{icons::Icons, layout, theme::Theme};
use ratatui::layout::Alignment;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Docker,
    System,
    Network,
    Help,
}

impl View {
    pub fn next(self) -> Self {
        match self {
            View::Docker => View::System,
            View::System => View::Network,
            View::Network => View::Help,
            View::Help => View::Docker,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            View::Docker => View::Help,
            View::System => View::Docker,
            View::Network => View::System,
            View::Help => View::Network,
        }
    }
}

pub fn render(frame: &mut Frame, app: &mut App) {
    let layout = layout::split(frame.area());
    let theme = Theme::default();

    let spinner_frames = ["|", "/", "-", "\\"];
    let spinner = spinner_frames[(app.tick_count as usize) % spinner_frames.len()];

    let header_block = theme.soft_block().title(Span::styled(
        format!(" {} eefoctui ", Icons::LOGO),
        theme.accent_style(),
    ));
    let active_view = match app.current_view {
        View::Docker => "Docker",
        View::System => "System",
        View::Network => "Network",
        View::Help => "Help",
    };
    let header_text = Line::from(vec![
        Span::styled(" Dashboard ", theme.accent_style()),
        Span::styled(format!(" {} ", Icons::ARROW_RIGHT), theme.secondary_style()),
        Span::styled(active_view, theme.secondary_style()),
        Span::styled(format!(" {} ", Icons::ARROW_RIGHT), theme.secondary_style()),
        Span::styled(format!("Status: {} Active", spinner), theme.matcha_style()),
    ]);
    let header = Paragraph::new(header_text)
        .block(header_block)
        .alignment(Alignment::Left)
        .style(theme.base());
    frame.render_widget(header, layout.header);

    render_sidebar(frame, layout.sidebar, &theme, app);

    match app.current_view {
        View::Docker => docker::render(frame, layout.main, &theme, app),
        View::System => system::render(frame, layout.main, &theme, app),
        View::Network => network::render(frame, layout.main, &theme, app),
        View::Help => help::render(frame, layout.main, &theme, app),
    }

    let footer_text = Line::from(vec![
        Span::styled(format!(" {} ", Icons::ARROW_LEFT), theme.accent_style()),
        Span::raw("/"),
        Span::styled(format!(" {} ", Icons::ARROW_RIGHT), theme.accent_style()),
        Span::raw("switch  "),
        Span::styled(format!(" {} ", Icons::ARROW_UP), theme.accent_style()),
        Span::raw("/"),
        Span::styled(format!(" {} ", Icons::ARROW_DOWN), theme.accent_style()),
        Span::raw("move  "),
        Span::styled(" q ", theme.accent_style()),
        Span::raw("quit  "),
        Span::styled(" ? ", theme.accent_style()),
        Span::raw("help"),
    ]);
    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .block(theme.soft_block())
        .style(theme.base());
    frame.render_widget(footer, layout.footer);
}

fn render_sidebar(frame: &mut Frame, area: ratatui::layout::Rect, theme: &Theme, app: &App) {
    use View::*;

    let items = [
        (Docker, format!("{} Docker", Icons::DOCKER)),
        (System, format!("{} System", Icons::SYSTEM)),
        (Network, format!("{} Network", Icons::NETWORK)),
        (Help, format!("{} Help", Icons::HELP)),
    ];

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        format!(" {} NAVIGATION", Icons::NAVIGATION),
        theme.secondary_style().add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    for (view, label) in items {
        let is_active = app.current_view == view;
        let bullet = if is_active {
            format!("{} ", Icons::BULLET)
        } else {
            "  ".to_string()
        };

        let style = if is_active {
            theme.accent_style()
        } else {
            theme.base()
        };

        lines.push(Line::from(Span::styled(
            format!(" {}{}", bullet, label),
            style,
        )));
    }

    let sidebar = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .block(theme.soft_block())
        .style(theme.base());
    frame.render_widget(sidebar, area);
}

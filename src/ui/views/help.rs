use crate::app::App;
use crate::ui::icons::Icons;
use crate::ui::theme::Theme;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Borders, Paragraph};
use ratatui::Frame;

pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, _app: &mut App) {
    let lines = vec![
        Line::from(Span::styled("Global", theme.accent_style())),
        Line::from("  q          Quit"),
        Line::from("  Ctrl+< / > Switch views"),
        Line::from("  ^ / v      Move selection in lists"),
        Line::from("  Esc        Dismiss errors"),
        Line::from("  Ins        Cancel action / Detach"),
        Line::from(""),
        Line::from(Span::styled("Docker view", theme.accent_style())),
        Line::from("  Enter      Open in-app console"),
        Line::from("  Ins        Detach from console"),
        Line::from("  Backspace  Delete char in console"),
        Line::from("  < / >      Move cursor in input field"),
        Line::from("  o          Start selected container"),
        Line::from("  s          Stop selected container"),
        Line::from("  r          Restart selected"),
        Line::from("  Del        Delete selected (type name to confirm)"),
        Line::from("  Alt+^/v    Scroll container table (works in console too)"),
        Line::from(""),
        Line::from(Span::styled("Network view", theme.accent_style())),
        Line::from("  Tab        Switch between sub-tabs"),
        Line::from("  i          Start input for port scanner"),
        Line::from("  Tab        Switch input fields (in port scanner)"),
        Line::from("  p          Cycle port presets"),
        Line::from("  Enter      Start port scan"),
        Line::from(""),
        Line::from(Span::styled("System view", theme.accent_style())),
        Line::from("  Animated CPU bar chart, RAM gauge, disks and network summary"),
        Line::from(""),
        Line::from(Span::styled(
            format!(" {} Help & Keybindings ", Icons::HELP),
            theme.accent_style(),
        )),
        Line::from("  - Run this tool in a reasonably wide terminal for best layout."),
        Line::from("  - Over SSH, ensure your terminal supports truecolor for best appearance."),
    ];

    let help = Paragraph::new(lines)
        .style(theme.base())
        .block(theme.soft_block().borders(Borders::NONE));

    frame.render_widget(help, area);
}

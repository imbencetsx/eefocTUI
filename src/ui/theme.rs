use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders};

#[derive(Debug, Clone)]
pub struct Theme {
    pub accent: Color,    // Pale Orange
    pub secondary: Color, // Vibrant Orange
    pub matcha: Color,    // Matcha Green
    pub _warning: Color,
    pub error: Color,
}

impl Theme {
    pub fn default() -> Self {
        Self {
            accent: Color::Rgb(255, 178, 102),   // Pale Orange
            secondary: Color::Rgb(255, 140, 60), // Vibrant Orange
            matcha: Color::Rgb(155, 172, 132),   // Matcha Green
            _warning: Color::Rgb(250, 218, 94),  // Muted Yellow
            error: Color::Rgb(255, 105, 97),     // Soft Red
        }
    }

    pub fn base(&self) -> Style {
        Style::default()
    }

    pub fn accent_style(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    pub fn secondary_style(&self) -> Style {
        Style::default().fg(self.secondary)
    }

    pub fn matcha_style(&self) -> Style {
        Style::default().fg(self.matcha)
    }

    pub fn soft_block(&self) -> Block<'_> {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.secondary)) // Vibrant Orange borders
    }
}

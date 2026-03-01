use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct AppLayout {
    pub header: Rect,
    pub sidebar: Rect,
    pub main: Rect,
    pub footer: Rect,
}

pub fn split(frame: Rect) -> AppLayout {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3), // header
                Constraint::Min(5),    // body
                Constraint::Length(3), // footer
            ]
            .as_ref(),
        )
        .split(frame);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length(18), // sidebar
                Constraint::Min(10),    // main area
            ]
            .as_ref(),
        )
        .split(chunks[1]);

    AppLayout {
        header: chunks[0],
        sidebar: body_chunks[0],
        main: body_chunks[1],
        footer: chunks[2],
    }
}


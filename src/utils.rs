use ratatui::style::{Color, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType};
use std::time::Duration;

pub const TICK_RATE: Duration = Duration::from_millis(1000);
pub const MUTED: Color = Color::DarkGray;
pub const ACCENT: Color = Color::Cyan;

pub fn panel(title: &str) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(MUTED)
        .title(Line::from(format!(" {title} ")).fg(ACCENT).bold())
}

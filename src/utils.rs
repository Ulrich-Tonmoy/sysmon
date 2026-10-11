use ratatui::style::{Color, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Gauge};
use std::collections::VecDeque;
use std::time::Duration;

pub const TICK_RATE: Duration = Duration::from_millis(500);
pub const HISTORY_LEN: usize = 120;

pub const MUTED: Color = Color::DarkGray;
pub const ACCENT: Color = Color::Cyan;
pub const RAM_COLOR: Color = Color::Magenta;
pub const COLOR_DANGER: Color = Color::Red;
pub const COLOR_WARNING: Color = Color::Yellow;
pub const COLOR_NORMAL: Color = Color::Green;
pub const COLOR_GAUGE_BG: Color = Color::Black;
pub const TABLE_HIGHLIGHT_FOREGROUND: Color = Color::White;
pub const TABLE_HIGHLIGHT_BACKGROUND: Color = Color::Blue;

pub fn panel(title: &str) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(MUTED)
        .title(Line::from(format!(" {title} ")).fg(ACCENT).bold())
}

pub fn level_color(percent: f64) -> Color {
    if percent < 50.0 {
        COLOR_NORMAL
    } else if percent < 80.0 {
        COLOR_WARNING
    } else {
        COLOR_DANGER
    }
}

pub fn percent(used: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        used as f64 / total as f64 * 100.0
    }
}

pub fn usage_gauge(name: &str, used: u64, total: u64) -> Gauge<'static> {
    let pct = percent(used, total);
    let gb = |bytes: u64| bytes as f64 / 1024.0 / 1024.0 / 1024.0;

    Gauge::default()
        .ratio(pct / 100.0)
        .label(format!(
            "{name} {:.1} / {:.1} GB ({pct:.0})%",
            gb(used),
            gb(total),
        ))
        .gauge_style(Style::new().fg(level_color(pct)).bg(COLOR_GAUGE_BG))
        .use_unicode(true)
}

pub fn push_history(history: &mut VecDeque<f64>, value: f64) {
    history.push_back(value);

    if history.len() > HISTORY_LEN {
        history.pop_front();
    }
}

pub fn to_points(history: &VecDeque<f64>) -> Vec<(f64, f64)> {
    let offset = HISTORY_LEN - history.len();

    history
        .iter()
        .enumerate()
        .map(|(i, value)| ((offset + i) as f64, *value))
        .collect()
}

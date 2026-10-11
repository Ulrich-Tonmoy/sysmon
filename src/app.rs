use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use std::time::Instant;

use crate::utils::{ACCENT, MUTED, TICK_RATE, panel};

pub struct App {
    running: bool,
}

impl App {
    pub fn new() -> Self {
        Self { running: true }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        let mut last_tick = Instant::now();

        while self.running {
            terminal.draw(|frame| self.render(frame))?;

            let timeout = TICK_RATE.saturating_sub(last_tick.elapsed());

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key);
                }
            }

            if last_tick.elapsed() >= TICK_RATE {
                self.on_tick();
                last_tick = Instant::now();
            }
        }
        Ok(())
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.running = false,
            _ => {}
        }
    }

    pub fn on_tick(&mut self) {}

    pub fn render(&mut self, frame: &mut Frame) {
        let [header, top, history, processes, footer] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(frame.area());

        let [cpu, memory] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(top);

        self.render_header(frame, header);
        frame.render_widget(panel("CPU"), cpu);
        frame.render_widget(panel("Memory"), memory);
        frame.render_widget(panel("History"), history);
        frame.render_widget(panel("Processes"), processes);
        self.render_footer(frame, footer);
    }

    pub fn render_header(&mut self, frame: &mut Frame, area: Rect) {
        let sep = Span::styled(" | ", MUTED);
        let line = Line::from(vec![
            "System Monitor".bold().fg(ACCENT),
            sep,
            Span::raw("Monitor your CPU, Memory, Processes and more..."),
        ]);

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(MUTED);

        frame.render_widget(Paragraph::new(line).block(block), area);
    }

    pub fn render_footer(&mut self, frame: &mut Frame, area: Rect) {
        let key = |k: &'static str, desc: &'static str| {
            [
                Span::styled(format!(" {k} "), Style::new().fg(Color::Black).bg(ACCENT)),
                Span::styled(format!(" {desc}\t"), MUTED),
            ]
        };

        let spans: Vec<Span> = [
            key("q or esc", "quit"),
            // key("q", "quit"),
            // key("q", "quit"),
            // key("q", "quit"),
            // key("q", "quit"),
        ]
        .into_iter()
        .flatten()
        .collect();

        frame.render_widget(Line::from(spans), area);
    }
}

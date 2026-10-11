use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use std::time::Instant;
use sysinfo::{MINIMUM_CPU_UPDATE_INTERVAL, System};

use crate::utils::{ACCENT, MUTED, TICK_RATE, panel};

pub struct App {
    running: bool,
    system: System,
    host_name: String,
    os_name: String,
}

impl App {
    pub fn new() -> Self {
        let mut system = System::new_all();
        std::thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);
        system.refresh_cpu_usage();

        let mut app = Self {
            running: true,
            system,
            host_name: System::host_name().unwrap_or_else(|| "unknown".into()),
            os_name: System::long_os_version().unwrap_or_else(|| "unknown OS".into()),
        };

        app.on_tick();
        app
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

    pub fn on_tick(&mut self) {
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
    }

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
        self.render_cpu(frame, cpu);
        self.render_memory(frame, memory);
        frame.render_widget(panel("History"), history);
        frame.render_widget(panel("Processes"), processes);
        self.render_footer(frame, footer);
    }

    pub fn render_header(&mut self, frame: &mut Frame, area: Rect) {
        let sep = Span::styled(" | ", MUTED);
        let uptime = System::uptime();

        let line = Line::from(vec![
            "System Monitor".bold().fg(ACCENT),
            sep.clone(),
            self.host_name.clone().bold(),
            sep.clone(),
            Span::raw(format!("up {}h: {}m", uptime / 3600, uptime % 3600 / 60)),
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

    fn render_cpu(&mut self, frame: &mut Frame, area: Rect) {
        let average = self.system.global_cpu_usage();
        let lines: Vec<Line> = self
            .system
            .cpus()
            .iter()
            .enumerate()
            .map(|(i, cpu)| Line::from(format!("core {i:>2}\t{:>3.0}%", cpu.cpu_usage())))
            .collect();

        let block = panel(&format!("CPU {average:.0}%"));
        frame.render_widget(Paragraph::new(lines).block(block), area);
    }

    fn render_memory(&mut self, frame: &mut Frame, area: Rect) {
        let gb = |bytes: u64| bytes as f64 / 1024.0 / 1024.0 / 1024.0;

        let sys = &self.system;

        let lines = vec![
            Line::from(format!(
                "RAM {:.1}/{:.1} GB",
                gb(sys.used_memory()),
                gb(sys.total_memory()),
            )),
            Line::from(format!(
                "Swap {:.1}/{:.1} GB",
                gb(sys.used_swap()),
                gb(sys.total_swap()),
            )),
        ];

        frame.render_widget(Paragraph::new(lines).block(panel("Memory")), area);
    }
}

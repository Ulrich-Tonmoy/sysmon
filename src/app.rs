use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Axis, Block, BorderType, Chart, Dataset, GraphType, LineGauge, Paragraph, Row, Sparkline,
    Table, TableState,
};
use ratatui::{DefaultTerminal, Frame};
use std::collections::VecDeque;
use std::time::Instant;
use sysinfo::{Disks, MINIMUM_CPU_UPDATE_INTERVAL, ProcessesToUpdate, System};

use crate::utils::{
    ACCENT, HISTORY_LEN, MUTED, RAM_COLOR, TABLE_HIGHLIGHT_BACKGROUND, TABLE_HIGHLIGHT_FOREGROUND,
    TICK_RATE, level_color, panel, percent, push_history, to_points, usage_gauge,
};

#[derive(PartialEq, Eq)]
enum SortBy {
    Cpu,
    Memory,
}

struct ProcessRow {
    pid: u32,
    name: String,
    cpu: f32,
    memory_mb: u64,
}

pub struct App {
    running: bool,
    system: System,
    disks: Disks,
    host_name: String,
    os_name: String,
    cpu_history: VecDeque<f64>,
    ram_history: VecDeque<f64>,
    sort_by: SortBy,
    processes: Vec<ProcessRow>,
    table_state: TableState,
}

impl App {
    pub fn new() -> Self {
        let mut system = System::new_all();
        std::thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);
        system.refresh_cpu_usage();

        let mut app = Self {
            running: true,
            system,
            disks: Disks::new_with_refreshed_list(),
            host_name: System::host_name().unwrap_or_else(|| "unknown".into()),
            os_name: System::long_os_version().unwrap_or_else(|| "unknown OS".into()),
            cpu_history: VecDeque::with_capacity(HISTORY_LEN),
            ram_history: VecDeque::with_capacity(HISTORY_LEN),
            sort_by: SortBy::Cpu,
            processes: Vec::new(),
            table_state: TableState::default(),
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
            KeyCode::Char('j') | KeyCode::Down => self.table_state.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.table_state.select_previous(),
            KeyCode::Char('c') => {
                if self.sort_by != SortBy::Cpu {
                    self.sort_by = SortBy::Cpu;
                    self.sort_processes();
                }
            }
            KeyCode::Char('m') => {
                if self.sort_by != SortBy::Memory {
                    self.sort_by = SortBy::Memory;
                    self.sort_processes();
                }
            }
            _ => {}
        }
    }

    pub fn on_tick(&mut self) {
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
        self.disks.refresh(true);
        self.system.refresh_processes(ProcessesToUpdate::All, true);

        push_history(&mut self.cpu_history, self.system.global_cpu_usage() as f64);
        push_history(
            &mut self.ram_history,
            percent(self.system.used_memory(), self.system.total_memory()),
        );

        self.collect_processes();
    }

    fn collect_processes(&mut self) {
        let cores = self.system.cpus().len().max(1) as f32;
        self.processes = self
            .system
            .processes()
            .values()
            .map(|p| ProcessRow {
                pid: p.pid().as_u32(),
                name: p.name().to_string_lossy().into_owned(),
                cpu: p.cpu_usage() / cores,
                memory_mb: p.memory() / 1024 / 1024,
            })
            .collect();

        self.sort_processes();
    }

    fn sort_processes(&mut self) {
        match self.sort_by {
            SortBy::Cpu => self.processes.sort_by(|a, b| b.cpu.total_cmp(&a.cpu)),
            SortBy::Memory => self.processes.sort_by(|a, b| b.memory_mb.cmp(&a.memory_mb)),
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let core_rows = self.system.cpus().len().div_ceil(2) as u16;
        let top_height = (core_rows + 3).max(5) + 2;

        let [header, top, history, processes, footer] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(top_height),
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
        self.render_history(frame, history);
        self.render_processes(frame, processes);
        self.render_footer(frame, footer);
    }

    pub fn render_header(&mut self, frame: &mut Frame, area: Rect) {
        let sep = Span::styled(" | ", MUTED);
        let uptime = System::uptime();

        let line = Line::from(vec![
            "System Monitor".bold().fg(ACCENT),
            sep.clone(),
            self.os_name.clone().bold(),
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
            key("j or down", "navigate process"),
            key("k or up", "navigate process"),
            key("c", "sort process by cpu"),
            key("m", "sort process by memory"),
        ]
        .into_iter()
        .flatten()
        .collect();

        frame.render_widget(Line::from(spans), area);
    }

    fn render_cpu(&mut self, frame: &mut Frame, area: Rect) {
        let average = self.system.global_cpu_usage();

        let block = panel(&format!("CPU {average:.0}%"));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let core_rows = self.system.cpus().len().div_ceil(2) as u16;

        let [cores_area, spark_area] =
            Layout::vertical([Constraint::Length(core_rows), Constraint::Fill(1)])
                .spacing(2)
                .areas(inner);

        let [left, right] = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)])
            .spacing(2)
            .areas(cores_area);

        for (i, cpu) in self.system.cpus().iter().enumerate() {
            let usage = cpu.cpu_usage() as f64;
            let column = if i % 2 == 0 { left } else { right };
            let row = Rect {
                y: column.y + (i / 2) as u16,
                height: 1,
                ..column
            };

            let gauge = LineGauge::default()
                .ratio((usage / 100.0).clamp(0.0, 1.0))
                .label(format!("{:>2} {:>3.0}%", i + 1, usage))
                .filled_symbol("█")
                .unfilled_symbol("░")
                .filled_style(level_color(usage))
                .unfilled_style(MUTED);

            frame.render_widget(gauge, row);
        }

        let width = spark_area.width as usize;
        let skip = self.cpu_history.len().saturating_sub(width);
        let data: Vec<u64> = self
            .cpu_history
            .iter()
            .skip(skip)
            .map(|v| *v as u64)
            .collect();

        let sparkline = Sparkline::default().data(&data).max(100).style(ACCENT);
        frame.render_widget(sparkline, spark_area);
    }

    fn render_memory(&mut self, frame: &mut Frame, area: Rect) {
        let block = panel("Memory");
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let [ram_area, swap_area, disk_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .spacing(1)
        .areas(inner);

        let sys = &self.system;

        frame.render_widget(
            usage_gauge("RAM", sys.used_memory(), sys.total_memory()),
            ram_area,
        );
        frame.render_widget(
            usage_gauge("Swap", sys.used_swap(), sys.total_swap()),
            swap_area,
        );

        let disk_list = self.disks.list();
        let disk_count = disk_list.len();

        if disk_count > 0 {
            let constraints = vec![Constraint::Length(1); disk_count];
            let disk_chunks = Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints(constraints)
                .spacing(1)
                .split(disk_area);

            for (i, disk) in disk_list.iter().enumerate() {
                let total = disk.total_space();
                let used = total - disk.available_space();
                let label = format!("Disk {}", disk.mount_point().display());

                if let Some(target_chunk) = disk_chunks.get(i) {
                    frame.render_widget(usage_gauge(&label, used, total), *target_chunk);
                }
            }
        }
    }

    fn render_history(&self, frame: &mut Frame, area: Rect) {
        let cpu_points = to_points(&self.cpu_history);
        let ram_points = to_points(&self.ram_history);

        let datasets = vec![
            Dataset::default()
                .name("CPU")
                .marker(Marker::Braille)
                .graph_type(GraphType::Line)
                .style(ACCENT)
                .data(&cpu_points),
            Dataset::default()
                .name("RAM")
                .marker(Marker::Braille)
                .graph_type(GraphType::Line)
                .style(RAM_COLOR)
                .data(&ram_points),
        ];

        let x_axis = Axis::default()
            .bounds([0.0, (HISTORY_LEN - 1) as f64])
            .labels(["60s ago", "30s", "now"])
            .style(MUTED);
        let y_axis = Axis::default()
            .bounds([0.0, 100.0])
            .labels(["0%", "50%", "100%"])
            .style(MUTED);

        let legend = Line::from(vec![("CPU".fg(ACCENT)), ("/RAM".fg(RAM_COLOR))]).right_aligned();
        let block = panel("History 60s").title_top(legend);

        let chart = Chart::new(datasets)
            .block(block)
            .x_axis(x_axis)
            .y_axis(y_axis)
            .legend_position(None);

        frame.render_widget(chart, area);
    }

    fn render_processes(&mut self, frame: &mut Frame, area: Rect) {
        let sort_name = match self.sort_by {
            SortBy::Cpu => "CPU",
            SortBy::Memory => "Memory",
        };

        let rows = self.processes.iter().map(|p| {
            Row::new([
                p.pid.to_string(),
                p.name.clone().to_string(),
                format!("{:.1}", p.cpu),
                format!("{} MB", p.memory_mb),
            ])
        });

        let header =
            Row::new(["PID", "Name", "CPU %", "Memory"]).style(Style::new().fg(ACCENT).bold());
        let widths = [
            Constraint::Length(8),
            Constraint::Fill(1),
            Constraint::Length(7),
            Constraint::Length(10),
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .block(panel(&format!(
                "Processes sorted by {} ({})",
                sort_name,
                self.processes.len()
            )))
            .row_highlight_style(
                Style::new()
                    .bg(TABLE_HIGHLIGHT_BACKGROUND)
                    .fg(TABLE_HIGHLIGHT_FOREGROUND)
                    .bold(),
            )
            .highlight_symbol("󱞵 ");

        frame.render_stateful_widget(table, area, &mut self.table_state);
    }
}

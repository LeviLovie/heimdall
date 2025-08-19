mod panels;

use anyhow::{anyhow, Context, Result};
use crossterm::event::{self, KeyCode, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, List, ListDirection, ListState, Paragraph, StatefulWidget, Widget, Wrap,
    },
    DefaultTerminal,
};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::data::Data;
use heimdall::{log::RsLog, status::ThreadType};

pub fn start(data: Arc<Mutex<Data>>) -> Result<()> {
    color_eyre::install()
        .map_err(|r| anyhow!("{}", r))
        .context("Failed to install color_eyre")?;

    let app = App::new(data);

    let terminal = ratatui::init();
    app.run(terminal)
        .context("Failed to run the TUI terminal")?;
    ratatui::restore();

    Ok(())
}

struct App {
    data: Arc<Mutex<Data>>,
    should_exit: bool,
    logs_state: ListState,
    logs_scroll: usize,
    logs_amount: usize,
    logs_area_height: Arc<Mutex<usize>>,
}

impl App {
    pub fn new(data: Arc<Mutex<Data>>) -> Self {
        Self {
            data,
            should_exit: false,
            logs_state: ListState::default(),
            logs_scroll: 0,
            logs_amount: 0,
            logs_area_height: Arc::new(Mutex::new(0)),
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            self.update().context("Failed to update the app")?;
            self.pool_events().context("Failed to pool events")?;

            if self.should_exit {
                break;
            }
        }
        Ok(())
    }

    fn update(&mut self) -> Result<()> {
        if self
            .data
            .lock()
            .unwrap()
            .statuses
            .all_stopped_except(ThreadType::TUI)
        {
            self.should_exit = true;
        }
        self.logs_amount = self.data.lock().unwrap().storage.get_logs().len();

        if self.logs_state.selected().is_none() && self.logs_amount > 0 {
            self.logs_state.select(Some(0));
        } else if self.logs_state.selected().unwrap_or(0) >= self.logs_amount {
            self.logs_state
                .select(Some(self.logs_amount.saturating_sub(1)));
        }

        let visible_height = *self
            .logs_area_height
            .lock()
            .expect("Failed to lock logs_area_height");
        let selected = self.logs_state.selected().unwrap_or(0);
        if selected < self.logs_scroll {
            self.logs_scroll = selected;
        } else if selected >= self.logs_scroll + visible_height {
            self.logs_scroll = selected + 1 - visible_height;
        }

        Ok(())
    }

    fn pool_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(250)).context("Failed to poll event")? {
            if let event::Event::Key(key) = event::read().context("Failed to read event")? {
                match (key.modifiers, key.code) {
                    (KeyModifiers::NONE, KeyCode::Char('q')) => {
                        self.data.lock().unwrap().statuses.terminate_all();
                        return Ok(());
                    }
                    (KeyModifiers::NONE, KeyCode::Char('w')) => {
                        self.should_exit = true;
                        return Ok(());
                    }

                    (KeyModifiers::SHIFT, KeyCode::Down)
                    | (KeyModifiers::NONE, KeyCode::Char('g')) => {
                        self.logs_state.select(Some(0));
                    }
                    (KeyModifiers::SHIFT, KeyCode::Up)
                    | (KeyModifiers::SHIFT, KeyCode::Char('G')) => {
                        self.logs_state
                            .select(Some(self.logs_amount.saturating_sub(1)));
                    }

                    (KeyModifiers::NONE, KeyCode::Char('z')) => {
                        self.center_cursor();
                    }
                    // The list is rendered in the reverse order, so J and K should be swapped.
                    (KeyModifiers::NONE, KeyCode::Char('j'))
                    | (KeyModifiers::NONE, KeyCode::Down) => {
                        self.logs_state.select_previous();
                    }
                    (KeyModifiers::NONE, KeyCode::Char('k'))
                    | (KeyModifiers::NONE, KeyCode::Up) => {
                        self.logs_state.select_next();
                    }

                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn center_cursor(&mut self) {
        let visible_height = *self
            .logs_area_height
            .lock()
            .expect("Failed to lock logs_area_height");
        if visible_height == 0 {
            return;
        }

        let selected = self.logs_state.selected().unwrap_or(0);
        let middle = visible_height / 2;
        self.logs_scroll = selected.saturating_sub(middle);
        self.logs_scroll = self
            .logs_scroll
            .min(self.logs_amount.saturating_sub(visible_height));
    }

    fn get_log(&self, index: usize) -> Option<RsLog> {
        let data = self.data.lock().unwrap();
        let logs = data.storage.get_logs().iter().rev().collect::<Vec<_>>();
        logs.get(index).map(|log| (*log).clone())
    }

    fn get_visible_logs(&self, height: usize) -> Vec<Line> {
        let data = self.data.lock().unwrap();
        let logs = data.storage.get_logs().iter().rev().collect::<Vec<_>>();

        if logs.is_empty() {
            return vec![];
        }

        let start = self.logs_scroll;
        let end = std::cmp::min(start + height, logs.len());

        logs[start..end]
            .iter()
            .map(|log| {
                let mut spans = vec![
                    Span::styled(
                        format!("{}", log.ts.format("%H:%M:%S%.6f")),
                        Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(": ", Style::default().fg(Color::DarkGray)),
                    Span::raw(format!("{}", log.msg)),
                ];

                if !log.vars.is_empty() {
                    spans.push(Span::raw(" "));
                }
                for var in log.vars.iter() {
                    spans.extend(vec![
                        Span::styled(format!("{}", var.key), Style::default().fg(Color::Green)),
                        Span::styled("=", Style::default().fg(Color::DarkGray)),
                        Span::styled(format!("{}", var.val), Style::default().fg(Color::Yellow)),
                    ]);
                }

                Line::from(spans)
            })
            .collect()
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical_areas =
            Layout::vertical([Constraint::Length(5), Constraint::Min(3)]).split(area);
        let status_horizontal_area = vertical_areas[0];
        let logs_horizontal_area = vertical_areas[1];

        let logs_horizontal_areas = if self
            .get_log(self.logs_state.selected().unwrap_or(0))
            .is_some()
        {
            Layout::horizontal([Constraint::Fill(2), Constraint::Fill(1)])
        } else {
            Layout::horizontal([Constraint::Min(10), Constraint::Max(0)])
        }
        .split(logs_horizontal_area);
        let logs_area = logs_horizontal_areas[0];
        let info_area = logs_horizontal_areas[1];

        let status_horizontal_areas =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(30)])
                .split(status_horizontal_area);
        let status_area = status_horizontal_areas[0];
        let threads_area = status_horizontal_areas[1];

        {
            let status_block = Block::bordered()
                .title("Status")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded);

            let greeting =
                Paragraph::new(format!("{} Logs, q to quit", self.logs_amount)).block(status_block);
            greeting.render(status_area, buf);
        };

        {
            let threads_block = Block::bordered()
                .title("Threads")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded);

            let threads = self
                .data
                .lock()
                .unwrap()
                .statuses
                .get_all()
                .iter()
                .map(|(kind, status)| {
                    let status_color = match status {
                        heimdall::status::ThreadStatus::Running => Color::Green,
                        heimdall::status::ThreadStatus::Terminating => Color::Magenta,
                        heimdall::status::ThreadStatus::Stopped => Color::Yellow,
                        heimdall::status::ThreadStatus::Failed(_) => Color::Red,
                    };
                    Line::from(vec![
                        Span::styled(
                            format!("{:?}", kind),
                            Style::default()
                                .fg(Color::Blue)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(" => ", Style::default().fg(Color::DarkGray)),
                        Span::styled(format!("{:?}", status), Style::default().fg(status_color)),
                    ])
                })
                .collect::<Vec<Line>>();

            let threads_list = List::new(threads)
                .direction(ListDirection::BottomToTop)
                .block(threads_block)
                .highlight_style(Style::default().bg(Color::White).fg(Color::Black));
            Widget::render(threads_list, threads_area, buf);
        };

        {
            let logs_block = Block::bordered()
                .title("Logs")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded);
            *self
                .logs_area_height
                .lock()
                .expect("Failed to log logs_area_height mutex") =
                logs_block.inner(logs_area).height as usize;
            let visible_height = logs_area.height as usize;
            let visible_logs = self.get_visible_logs(visible_height);

            let mut logs_state = ListState::default();
            logs_state.select(Some(
                self.logs_state.selected().unwrap_or(0) - self.logs_scroll,
            ));

            let logs_list = List::new(visible_logs)
                .direction(ListDirection::BottomToTop)
                .block(logs_block)
                .highlight_style(Style::default().bg(Color::White).fg(Color::Black));
            StatefulWidget::render(logs_list, logs_area, buf, &mut logs_state);
        };

        if let Some(log) = self.get_log(self.logs_state.selected().unwrap_or(0)) {
            let info_block = Block::bordered()
                .title("Info")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded);

            let mut lines = vec![
                Line::from(vec![
                    Span::styled("at ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{}", log.ts),
                        Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("from ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{} ", log.context.app),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("v", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{} ", log.context.version),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("on ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{} ", log.ip),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("pid ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{} ", log.context.pid),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(""),
                Line::from(log.msg.to_string()),
            ];
            if !log.vars.is_empty() {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![Span::styled(
                    "with",
                    Style::default().fg(Color::DarkGray),
                )]));
                lines.extend(log.vars.iter().map(|var| {
                    Line::from(vec![
                        Span::raw("  "),
                        Span::styled(
                            format!("{}: ", var.key),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::styled(var.val.to_string(), Style::default().fg(Color::Green)),
                    ])
                }));
            }
            let paragraph = Paragraph::new(lines)
                .wrap(Wrap { trim: false })
                .block(info_block);
            paragraph.render(info_area, buf);
        }
    }
}

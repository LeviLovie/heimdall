use anyhow::{anyhow, Context, Result};
use crossterm::event::{self, KeyCode};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, List, ListDirection, ListState, Paragraph, StatefulWidget, Widget,
    },
    DefaultTerminal,
};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use heimdall::{log::RsLog, storage::Storage};

pub fn start(storage: Arc<Mutex<Storage>>) -> Result<()> {
    color_eyre::install()
        .map_err(|r| anyhow!("{}", r))
        .context("Failed to install color_eyre")?;

    let app = App::new(storage.clone());

    let terminal = ratatui::init();
    app.run(terminal)
        .context("Failed to run the TUI terminal")?;
    ratatui::restore();

    Ok(())
}

struct App {
    storage: Arc<Mutex<Storage>>,
    logs_amount: usize,
    should_exit: bool,
    show_info: bool,
    logs_state: ListState,
}

impl App {
    pub fn new(storage: Arc<Mutex<Storage>>) -> Self {
        Self {
            storage,
            logs_amount: 0,
            should_exit: false,
            show_info: true,
            logs_state: ListState::default(),
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            self.pool_events().context("Failed to pool events")?;
            self.update().context("Failed to update the app")?;

            if self.should_exit {
                break;
            }
        }
        Ok(())
    }

    fn update(&mut self) -> Result<()> {
        {
            let storage = self.storage.lock().expect("Failed to lock storage");
            let logs = storage.get_logs();
            self.logs_amount = logs.len();
        };

        if self.logs_state.selected().is_none() && self.logs_amount > 0 {
            self.logs_state.select(Some(0));
        } else {
            if self.logs_state.selected().unwrap_or(0) >= self.logs_amount {
                self.logs_state
                    .select(Some(self.logs_amount.saturating_sub(1)));
            }
        }

        Ok(())
    }

    fn pool_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(250)).context("Failed to poll event")? {
            match event::read().context("Failed to read event")? {
                event::Event::Key(key) => match key.code {
                    KeyCode::Char('q') => {
                        self.should_exit = true;
                        return Ok(());
                    }
                    // The list is rendered in the reverse order, so J and K should be swapped.
                    KeyCode::Char('j') => {
                        self.logs_state.select_previous();
                    }
                    KeyCode::Char('k') => {
                        self.logs_state.select_next();
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        Ok(())
    }

    fn get_log(&self, index: usize) -> Option<RsLog> {
        let storage = self.storage.lock().expect("Failed to lock storage");
        let logs = storage.get_logs().clone().iter().rev().collect::<Vec<_>>();
        if index < logs.len() {
            Some(logs[index].clone())
        } else {
            None
        }
    }

    fn get_logs_list(&self) -> List {
        let storage = self.storage.lock().expect("Failed to lock storage");
        let logs = storage
            .get_logs()
            .iter()
            .map(|log| format!("{}", log))
            .rev()
            .collect::<Vec<_>>();
        List::new(logs).direction(ListDirection::BottomToTop)
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical_areas =
            Layout::vertical([Constraint::Length(5), Constraint::Min(3)]).split(area);
        let status_area = vertical_areas[0];
        let horizontal_area = vertical_areas[1];

        let horizontal_areas = if self
            .get_log(self.logs_state.selected().unwrap_or(0))
            .is_some()
        {
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)])
        } else {
            Layout::horizontal([Constraint::Min(10), Constraint::Max(0)])
        }
        .split(horizontal_area);
        let logs_area = horizontal_areas[0];
        let info_area = horizontal_areas[1];

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
            let logs_block = Block::bordered()
                .title("Logs")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded);

            let logs_list = self
                .get_logs_list()
                .block(logs_block)
                .highlight_style(Style::default().bg(Color::White).fg(Color::Black));
            let mut logs_state = self.logs_state.clone();
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
                        format!("{} ", log.context.machine),
                        Style::default().fg(Color::Green),
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
                Line::from(format!("{}", log.msg)),
            ];
            if log.vars.len() > 0 {
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
                        Span::styled(format!("{}", var.val), Style::default().fg(Color::Green)),
                    ])
                }));
            }
            let paragraph = Paragraph::new(lines).block(info_block);
            paragraph.render(info_area, buf);
        }
    }
}

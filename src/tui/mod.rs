mod panels;

use panels::prelude::*;

use anyhow::{anyhow, Context, Result};
use crossterm::event::{self, KeyCode, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::Widget,
    DefaultTerminal,
};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::data::Data;
use heimdall::status::ThreadType;

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
    logs_panel: LogsPanel,
}

impl App {
    pub fn new(data: Arc<Mutex<Data>>) -> Self {
        Self {
            data: data.clone(),
            should_exit: false,
            logs_panel: LogsPanel::new(data.clone()),
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
        if self
            .data
            .lock()
            .unwrap()
            .statuses
            .all_stopped_except(ThreadType::TUI)
        {
            self.should_exit = true;
        }

        self.logs_panel.update();

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

                    // The list is rendered in the reverse order, so J and K should be swapped.
                    (KeyModifiers::NONE, KeyCode::Char('j'))
                    | (KeyModifiers::NONE, KeyCode::Down) => {
                        self.logs_panel.logs_state.select_previous();
                    }
                    (KeyModifiers::NONE, KeyCode::Char('k'))
                    | (KeyModifiers::NONE, KeyCode::Up) => {
                        self.logs_panel.logs_state.select_next();
                    }
                    (KeyModifiers::SHIFT, KeyCode::Down)
                    | (KeyModifiers::NONE, KeyCode::Char('g')) => {
                        self.logs_panel
                            .logs_state
                            .select(Some(self.logs_panel.logs_amount.saturating_sub(1)));
                    }
                    (KeyModifiers::SHIFT, KeyCode::Up)
                    | (KeyModifiers::SHIFT, KeyCode::Char('G')) => {
                        self.logs_panel.logs_state.select(Some(0));
                    }

                    _ => {}
                }
            }
        }
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [statuses, data] =
            Layout::vertical([Constraint::Length(5), Constraint::Min(3)]).areas(area);

        let [status, threads] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(30)]).areas(statuses);

        let log = self
            .logs_panel
            .visible_logs
            .iter()
            .find(|(id, _)| id == &self.logs_panel.logs_state.selected().unwrap_or(0))
            .map(|(_, log)| log.clone());

        let [logs, info] = if log.is_some() {
            Layout::horizontal([Constraint::Fill(2), Constraint::Fill(1)])
        } else {
            Layout::horizontal([Constraint::Min(10), Constraint::Max(0)])
        }
        .areas(data);

        StatusPanel::from(self.logs_panel.logs_amount).render(status, buf);
        ThreadsPanel::from(self.data.clone()).render(threads, buf);
        self.logs_panel.render(logs, buf);

        if let Some(log) = log {
            InfoPanel::from(log).render(info, buf);
        }
    }
}

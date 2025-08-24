mod panels;
mod popups;

use anyhow::{anyhow, Context, Result};
use crossterm::event::{self, KeyCode, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::Widget,
    DefaultTerminal,
};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
    time::Duration,
};

use panels::prelude::*;
use popups::prelude::*;

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

struct AppData {
    data: Arc<Mutex<Data>>,
    should_exit: bool,
    logs_panel: LogsPanel,
    popups: Vec<Box<dyn popups::Popup>>,
}

struct App {
    app_data: Rc<RefCell<AppData>>,
}

impl App {
    pub fn new(data: Arc<Mutex<Data>>) -> Self {
        Self {
            app_data: Rc::new(RefCell::new(AppData {
                data: data.clone(),
                should_exit: false,
                logs_panel: LogsPanel::new(data.clone()),
                popups: vec![],
            })),
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            self.app_data
                .borrow_mut()
                .popups
                .sort_by(|a, b| b.priority().cmp(&a.priority()));

            self.pool_events().context("Failed to pool events")?;
            self.update().context("Failed to update the app")?;
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;

            if self.app_data.borrow().should_exit {
                break;
            }
        }
        Ok(())
    }

    fn update(&mut self) -> Result<()> {
        if self
            .app_data
            .borrow()
            .data
            .lock()
            .unwrap()
            .statuses
            .all_stopped_except(ThreadType::TUI)
        {
            self.app_data.borrow_mut().should_exit = true;
        }

        // TODO: Figure out a way to avoid double borrow (remove unsafe code)
        {
            let popup_ptr: Option<*mut Box<dyn Popup>> = {
                let mut app_data = self.app_data.borrow_mut();
                app_data.popups.get_mut(0).map(|p| p as *mut Box<dyn Popup>)
            };

            // SAFETY: we know popup_ptr points to a valid object
            if let Some(popup_ptr) = popup_ptr {
                let mut app_data = self.app_data.borrow_mut(); // re-borrow after releasing first borrow
                unsafe {
                    (*popup_ptr).update(&mut app_data);
                }
            }
        }

        self.app_data.borrow_mut().logs_panel.update();

        Ok(())
    }

    fn pool_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(250)).context("Failed to poll event")? {
            if let event::Event::Key(key) = event::read().context("Failed to read event")? {
                match (key.modifiers, key.code) {
                    (KeyModifiers::NONE, KeyCode::Char('q')) => {
                        self.app_data
                            .borrow_mut()
                            .popups
                            .push(Box::new(ExitPopup::new()));
                    }
                    _ => {}
                }

                if let Some(popup) = self.app_data.borrow_mut().popups.first_mut() {
                    popup.on_event(key);
                } else {
                    match (key.modifiers, key.code) {
                        (KeyModifiers::NONE, KeyCode::Char('w')) => {
                            self.app_data.borrow_mut().should_exit = true;
                            return Ok(());
                        }

                        // The list is rendered in the reverse order, so J and K should be swapped.
                        (KeyModifiers::NONE, KeyCode::Char('j'))
                        | (KeyModifiers::NONE, KeyCode::Down) => {
                            self.app_data
                                .borrow_mut()
                                .logs_panel
                                .logs_state
                                .select_previous();
                        }
                        (KeyModifiers::NONE, KeyCode::Char('k'))
                        | (KeyModifiers::NONE, KeyCode::Up) => {
                            self.app_data
                                .borrow_mut()
                                .logs_panel
                                .logs_state
                                .select_next();
                        }
                        (KeyModifiers::SHIFT, KeyCode::Down)
                        | (KeyModifiers::NONE, KeyCode::Char('g')) => {
                            self.app_data
                                .borrow_mut()
                                .logs_panel
                                .logs_state
                                .select(Some(
                                    self.app_data
                                        .borrow_mut()
                                        .logs_panel
                                        .logs_amount
                                        .saturating_sub(1),
                                ));
                        }
                        (KeyModifiers::SHIFT, KeyCode::Up)
                        | (KeyModifiers::SHIFT, KeyCode::Char('G')) => {
                            self.app_data
                                .borrow_mut()
                                .logs_panel
                                .logs_state
                                .select(Some(0));
                        }

                        _ => {}
                    }
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
            .app_data
            .borrow()
            .logs_panel
            .visible_logs
            .iter()
            .find(|(id, _)| {
                id == &self
                    .app_data
                    .borrow()
                    .logs_panel
                    .logs_state
                    .selected()
                    .unwrap_or(0)
            })
            .map(|(_, log)| log.clone());

        let [logs, info] = if log.is_some() {
            Layout::horizontal([Constraint::Fill(2), Constraint::Fill(1)])
        } else {
            Layout::horizontal([Constraint::Min(10), Constraint::Max(0)])
        }
        .areas(data);

        StatusPanel::from(self.app_data.borrow().logs_panel.logs_amount).render(status, buf);
        ThreadsPanel::from(self.app_data.borrow().data.clone()).render(threads, buf);
        self.app_data.borrow().logs_panel.render(logs, buf);

        if let Some(log) = log {
            InfoPanel::from(log).render(info, buf);
        }

        for popup in &self.app_data.borrow().popups {
            let area = popup.area(area);
            popup.render(area, buf);
        }
    }
}

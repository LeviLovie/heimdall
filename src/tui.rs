use anyhow::{anyhow, Context, Result};
use crossterm::event::{self, KeyCode};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Block, BorderType, List, ListDirection, ListItem, Paragraph, Widget},
    DefaultTerminal,
};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use heimdall::storage::Storage;

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
}

impl App {
    pub fn new(storage: Arc<Mutex<Storage>>) -> Self {
        Self { storage }
    }

    pub fn run(self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            if Self::should_quit().context("Failed to check if should quit")? {
                break;
            }
        }
        Ok(())
    }

    fn should_quit() -> Result<bool> {
        if event::poll(Duration::from_millis(250)).context("Failed to pull event")? {
            let q_pressed = event::read()
                .context("Failed to read event")?
                .as_key_press_event()
                .is_some_and(|key| key.code == KeyCode::Char('q'));
            return Ok(q_pressed);
        }
        Ok(false)
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(3)])
            .split(area);

        {
            let status_block = Block::bordered()
                .title("Status")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded);
            let greeting = Paragraph::new("Hello World! (press 'q' to quit)").block(status_block);
            greeting.render(chunks[0], buf);
        };

        {
            let logs_block = Block::bordered()
                .title("Logs")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded);

            let storage = self.storage.lock().expect("Failed to lock storage");
            let items: Vec<ListItem> = storage
                .get_logs()
                .iter()
                .rev()
                .map(|log| ListItem::new(format!("{}", log)))
                .collect();
            let logs_list = List::new(items)
                .block(logs_block)
                .direction(ListDirection::BottomToTop);
            logs_list.render(chunks[1], buf);
        };
    }
}

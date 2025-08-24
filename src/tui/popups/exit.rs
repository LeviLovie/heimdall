use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Widget},
};

use super::Popup;
use crate::tui::AppData;

pub struct ExitPopup {
    can_exit: bool,
    exit: bool,
}

impl ExitPopup {
    pub fn new() -> Self {
        Self {
            can_exit: false,
            exit: false,
        }
    }
}

impl Popup for ExitPopup {
    fn priority(&self) -> i32 {
        i32::MAX
    }

    fn area(&self, global_area: Rect) -> Rect {
        let width = 40;
        let height = 7;
        let x = (global_area.width.saturating_sub(width)) / 2;
        let y = (global_area.height.saturating_sub(height)) / 2;
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    fn on_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.can_exit = true;
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                self.exit = true;
            }
            _ => {}
        }
    }

    fn render(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let block = Block::bordered()
            .title("Exit Confirmation")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .style(Style::default().bg(Color::Red));

        Paragraph::new(vec![
            Line::from(""),
            Line::from("Are you sure you want to exit?"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Y", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" / "),
                Span::styled("N", Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
        ])
        .centered()
        .block(block)
        .render(area, buf);
    }

    fn update(&mut self, data: &mut AppData) -> bool {
        if self.can_exit {
            data.data.lock().unwrap().statuses.terminate_all();
        }
        self.exit
    }
}

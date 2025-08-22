use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, List, ListDirection, Widget},
};
use std::sync::{Arc, Mutex};

use super::Panel;
use crate::data::Data;

pub struct ThreadsPanel {
    pub data: Arc<Mutex<Data>>,
}

impl ThreadsPanel {
    pub fn from(data: Arc<Mutex<Data>>) -> Self {
        Self { data }
    }
}

impl Panel for ThreadsPanel {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
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
            .block(block)
            .highlight_style(Style::default().bg(Color::White).fg(Color::Black));
        Widget::render(threads_list, area, buf);
    }
}

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, List, ListDirection, ListState, StatefulWidget},
};
use std::sync::{Arc, Mutex};

use super::Panel;
use crate::data::Data;

pub struct LogsPanel {
    pub data: Arc<Mutex<Data>>,
    pub area_height: Arc<Mutex<usize>>,
    pub logs_state: ListState,
    pub logs_scroll: usize,
}

impl LogsPanel {
    pub fn new(data: Arc<Mutex<Data>>) -> Self {
        Self {
            data,
            area_height: Arc::new(Mutex::new(0)),
            logs_scroll: 0,
            logs_state: ListState::default(),
        }
    }

    pub fn update(&mut self, logs_amount: usize) {
        if self.logs_state.selected().is_none() && logs_amount > 0 {
            self.logs_state.select(Some(1));
        } else if self.logs_state.selected().unwrap_or(0) >= logs_amount {
            self.logs_state.select(Some(logs_amount.saturating_sub(1)));
        }

        let visible_height = *self
            .area_height
            .lock()
            .expect("Failed to lock logs_area_height");
        let selected = self.logs_state.selected().unwrap_or(0);
        if selected < self.logs_scroll {
            self.logs_scroll = selected;
        } else if selected >= self.logs_scroll + visible_height {
            self.logs_scroll = selected + 1 - visible_height;
        }
    }

    pub fn center_cursor(&mut self, logs_amount: usize) {
        let visible_height = *self
            .area_height
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
            .min(logs_amount.saturating_sub(visible_height));
    }

    fn get_visible_logs(&self, height: usize) -> Vec<Line<'_>> {
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

impl Panel for LogsPanel {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title("Logs")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        *self
            .area_height
            .lock()
            .expect("Failed to log area_height mutex") = block.inner(area).height as usize;
        let visible_height = area.height as usize;
        let visible_logs = self.get_visible_logs(visible_height);

        let mut logs_state = ListState::default();
        logs_state.select(Some(
            self.logs_state
                .selected()
                .unwrap_or(0)
                .saturating_sub(self.logs_scroll),
        ));

        let logs_list = List::new(visible_logs)
            .direction(ListDirection::BottomToTop)
            .block(block)
            .highlight_style(Style::default().bg(Color::White).fg(Color::Black));
        StatefulWidget::render(logs_list, area, buf, &mut logs_state);
    }
}

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
use heimdall::log::RsLog;

pub struct LogsPanel {
    pub data: Arc<Mutex<Data>>,
    pub area_height: Arc<Mutex<usize>>,
    pub logs_state: ListState,
    pub logs_scroll: usize,
    pub logs_amount: usize,
    pub visible_logs: Vec<(usize, RsLog)>,
    pub updated: bool,
}

impl LogsPanel {
    pub fn new(data: Arc<Mutex<Data>>) -> Self {
        Self {
            data,
            area_height: Arc::new(Mutex::new(0)),
            logs_scroll: 0,
            logs_state: ListState::default(),
            logs_amount: 0,
            visible_logs: vec![],
            updated: false,
        }
    }

    pub fn update(&mut self) {
        let data = self.data.lock().unwrap();

        let total_logs = data.storage.logs_amount();
        if total_logs == 0 {
            self.logs_state.select(None);
            self.visible_logs.clear();
            return;
        }
        if self.logs_state.selected().is_none() {
            self.logs_state.select(Some(0));
        }

        let selected = self.logs_state.selected().unwrap();

        let visible_height = *self.area_height.lock().unwrap();
        if visible_height > 0 {
            let middle = visible_height / 2;
            self.logs_scroll = selected.saturating_sub(middle);
            self.logs_scroll = self
                .logs_scroll
                .min(total_logs.saturating_sub(visible_height));
        }

        let chunk_size = 64;
        let area_height = *self.area_height.lock().unwrap();
        let chunk_start = (self.logs_scroll / chunk_size) * chunk_size;

        self.visible_logs = data
            .storage
            .get_visible_logs(chunk_start, chunk_size + area_height)
            .unwrap_or_default();

        self.logs_amount = total_logs;
        self.updated = false;
    }

    fn get_visible_logs(&self) -> Vec<Line<'_>> {
        let scroll_offset_in_chunk = self.logs_scroll % 64;
        let visible_height = *self.area_height.lock().unwrap();

        let logs_slice = &self.visible_logs[scroll_offset_in_chunk
            ..self
                .visible_logs
                .len()
                .min(scroll_offset_in_chunk + visible_height)];

        if logs_slice.is_empty() {
            return vec![Line::from(Span::styled(
                "No logs yet",
                Style::default().add_modifier(Modifier::ITALIC),
            ))];
        }

        logs_slice
            .iter()
            .map(|log| {
                let mut spans = vec![
                    Span::styled(
                        format!("{}", log.1.ts.format("%H:%M:%S%.6f")),
                        Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(": ", Style::default().fg(Color::DarkGray)),
                    Span::raw(format!("{}", log.1.msg)),
                ];

                for var in &log.1.vars {
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled(&var.key, Style::default().fg(Color::Green)));
                    spans.push(Span::styled("=", Style::default().fg(Color::DarkGray)));
                    spans.push(Span::styled(&var.val, Style::default().fg(Color::Yellow)));
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
        let visible_logs = self.get_visible_logs();

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

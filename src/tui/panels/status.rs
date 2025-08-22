use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, BorderType, Paragraph, Widget},
};

use super::Panel;

pub struct StatusPanel {
    pub logs_amount: usize,
}

impl StatusPanel {
    pub fn from(logs_amount: usize) -> Self {
        Self { logs_amount }
    }
}

impl Panel for StatusPanel {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title("Status")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        let paragraph = Paragraph::new(vec![
            Line::from(format!("Logs amount: {}", self.logs_amount)),
            Line::from("Press q to quit"),
        ]);
        paragraph
            .block(block)
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .render(area, buf);
    }
}

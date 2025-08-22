use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Widget, Wrap},
};

use super::Panel;
use heimdall::log::RsLog;

pub struct InfoPanel {
    pub log: RsLog,
}

impl InfoPanel {
    pub fn from(log: RsLog) -> Self {
        Self { log }
    }
}

impl Panel for InfoPanel {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title("Info")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        let mut lines = vec![
            Line::from(vec![
                Span::styled("at ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{}", self.log.ts),
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("from ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} ", self.log.context.app),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("v", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} ", self.log.context.version),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("on ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} ", self.log.ip),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("pid ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} ", self.log.context.pid),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(self.log.msg.to_string()),
        ];
        if !self.log.vars.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "with",
                Style::default().fg(Color::DarkGray),
            )]));
            lines.extend(self.log.vars.iter().map(|var| {
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
            .block(block);
        paragraph.render(area, buf);
    }
}

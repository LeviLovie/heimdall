mod info;
mod logs;
mod status;
mod threads;

use ratatui::{buffer::Buffer, layout::Rect};

pub trait Panel {
    fn render(&self, area: Rect, buf: &mut Buffer);
}

pub mod prelude {
    pub use super::{
        info::InfoPanel, logs::LogsPanel, status::StatusPanel, threads::ThreadsPanel, Panel,
    };
}

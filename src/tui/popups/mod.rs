mod exit;

pub mod prelude {
    pub use super::{exit::ExitPopup, Popup};
}

use crossterm::event;
use ratatui::{buffer::Buffer, layout::Rect};

use crate::tui::AppData;

pub trait Popup {
    fn priority(&self) -> i32;
    fn area(&self, global_area: Rect) -> Rect;
    fn render(&self, area: Rect, buf: &mut Buffer);
    fn update(&mut self, app_data: &mut AppData) -> bool; // is_finished
    fn on_event(&mut self, key: event::KeyEvent);
}

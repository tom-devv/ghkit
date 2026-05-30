pub mod cadence;
pub mod overview;

use crossterm::event::KeyCode;
use ratatui::{Frame, layout::Rect};

use crate::tui::state::State;

pub trait RenderMetric {
    fn render(&self, frame: &mut Frame, area: Rect, state: &State);

    fn update(&mut self, key: KeyCode);
}

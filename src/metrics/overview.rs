use ratatui::widgets::{Block, Borders};

use crate::{metrics::RenderMetric, tui::state::State};

#[derive(Default)]
pub struct OverviewMetric {}

impl RenderMetric for OverviewMetric {
    fn update(&mut self, _key: crossterm::event::KeyCode) {}

    fn render(
        &self,
        frame: &mut ratatui::prelude::Frame,
        area: ratatui::prelude::Rect,
        _state: &State,
    ) {
        let block_widg = Block::default()
            .title("Overview")
            .title_alignment(ratatui::layout::HorizontalAlignment::Center)
            .borders(Borders::ALL);

        frame.render_widget(block_widg, area);
    }
}

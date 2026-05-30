use ratatui::widgets::{Block, Borders};

pub struct OverviewPage {
    pub data: OverviewData,
}

#[derive(Default)]
pub struct OverviewData {}

impl OverviewPage {
    pub fn new(data: OverviewData) -> Self {
        Self { data }
    }

    pub fn render(&self, frame: &mut ratatui::prelude::Frame, area: ratatui::prelude::Rect) {
        let block_widg = Block::default()
            .title("Overview")
            .title_alignment(ratatui::layout::HorizontalAlignment::Center)
            .borders(Borders::ALL);

        frame.render_widget(block_widg, area);
    }
}

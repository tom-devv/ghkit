use crate::metrics::cadence::CadenceMetric;
use crate::metrics::overview::OverviewMetric;
use crate::tui::ui::Page;

#[derive(Default)]
pub struct State {
    pub is_quit: bool,
    pub loading: bool,
    pub page: Page,
    pub overview: Option<OverviewMetric>,
    pub cadence: Option<CadenceMetric>,
}

impl State {
    //By default new stats will be loading
    pub fn new() -> State {
        State {
            loading: true,
            ..Default::default()
        }
    }

    pub fn tab(&mut self) {
        let next_page = match self.page {
            Page::Overview => Page::Cadence,
            Page::Cadence => Page::Todo,
            Page::Todo => Page::Overview,
        };
        self.page = next_page;
    }
}

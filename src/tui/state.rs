use crossterm::event::{KeyCode, KeyEvent};

use crate::git::kit::GRepo;
use crate::metrics::cadence::{CadenceData, CadencePage};
use crate::metrics::overview::{OverviewData, OverviewPage};

use crate::error::Result;
use crate::tui::page::Page;

pub struct TuiState {
    pub is_quit: bool,
    pub loading: bool,
    pub active_page: Page,
    pub overview: OverviewPage,
    pub cadence: CadencePage,
}

impl TuiState {
    //By default new stats will be loading
    pub fn new(repo: &GRepo) -> Result<TuiState> {
        let cadence_data = CadenceData::full_report(repo)?;
        let overview_data = OverviewData::default();
        Ok(TuiState {
            is_quit: false,
            loading: false,
            active_page: Page::default(),
            overview: OverviewPage::new(overview_data),
            cadence: CadencePage::new(cadence_data),
        })
    }

    pub fn next_tab(&mut self) {
        let next_page = match self.active_page {
            Page::Overview => Page::Cadence,
            Page::Cadence => Page::Todo,
            Page::Todo => Page::Overview,
        };
        self.active_page = next_page;
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.is_quit = true,
            KeyCode::Tab => self.next_tab(),

            _ => match self.active_page {
                Page::Cadence => self.cadence.handle_key(key),
                // Page::Overview => self.overview.handle_key(key.code),
                Page::Todo => {}
                _ => {}
            },
        }
    }
}

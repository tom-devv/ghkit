
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum Page {
    #[default]
    Overview = 0, // Numbering is for easy recogonition of order on the top bar
    Cadence = 1,
    Todo = 2,
}

impl Page {
    pub const ALL: [Page; 3] = [Page::Overview, Page::Cadence, Page::Todo];

    pub fn to_tab(&self) -> &'static str {
        match self {
            Page::Overview => "Overview",
            Page::Cadence => "Cadence",
            Page::Todo => "Todo",
        }
    }

    pub fn size() -> usize {
        Self::ALL.len()
    }
}

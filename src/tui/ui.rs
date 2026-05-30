use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Tabs},
};

use crate::tui::{page::Page, state::TuiState};

pub fn render(frame: &mut Frame, state: &mut TuiState) {
    let chunks = Layout::vertical([Constraint::Length(Page::size() as u16), Constraint::Min(0)])
        .horizontal_margin(2)
        .vertical_margin(1)
        .split(frame.area());

    render_tabs(frame, &state, chunks[0]);

    match state.active_page {
        Page::Overview => state.overview.render(frame, chunks[1]),
        Page::Cadence => state.cadence.render(frame, chunks[1]),
        Page::Todo => {}
    }
}

fn render_tabs(frame: &mut Frame, state: &TuiState, chunk: Rect) {
    let nav_block = Block::bordered()
        .border_style(Color::Gray)
        .title("< Ghkit: {Version} >")
        .title_alignment(ratatui::layout::HorizontalAlignment::Center);

    let nav_tabs = nav(state).block(nav_block);

    frame.render_widget(nav_tabs, chunk);
}

pub fn draw_placeholder(frame: &mut Frame, area: Rect, label: &str, color: Color) {
    let placeholder = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color))
        .title(format!(" {} ({}x{}) ", label, area.width, area.height))
        .title_style(Style::default().fg(color));

    frame.render_widget(placeholder, area);
}

pub fn nav(state: &TuiState) -> Tabs<'static> {
    let tab_titles = Page::ALL.iter().map(|page| page.to_tab());
    let tabs = Tabs::new(tab_titles)
        .select(state.active_page as usize)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::White),
        );

    tabs
}

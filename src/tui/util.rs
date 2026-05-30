use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders},
};

pub fn draw_placeholder(frame: &mut Frame, area: Rect, label: &str, color: Color) {
    let placeholder = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color))
        .title(format!(" {} ({}x{}) ", label, area.width, area.height))
        .title_style(Style::default().fg(color));

    frame.render_widget(placeholder, area);
}

use std::collections::HashMap;

use chrono::{DateTime, TimeDelta, Utc};
use crossterm::event::KeyCode;

use crate::{
    git::kit::GRepo,
    tui::{state::State, util},
};
#[derive(Debug)]
pub struct CadenceMetric {
    pub global_commits_per_day: f32,
    pub author_commits_per_day: HashMap<String, f32>,
    pub selected_index: usize,
}

impl CadenceMetric {
    pub fn author_commits_per_day(repo: &GRepo, email: &str) -> Result<f32, git2::Error> {
        let commit_dates: Vec<DateTime<Utc>> = repo
            .get_author_commits(email)?
            .filter_map(|commit| DateTime::from_timestamp_secs(commit.time().seconds()))
            .collect();

        Ok(commits_per_day(&commit_dates))
    }

    pub fn global_commits_per_day(repo: &GRepo) -> Result<f32, git2::Error> {
        let commit_dates: Vec<DateTime<Utc>> = repo
            .iter_commits()?
            .filter_map(|commit| DateTime::from_timestamp_secs(commit.time().seconds()))
            .collect();

        Ok(commits_per_day(&commit_dates))
    }

    pub fn full_report(repo: &GRepo) -> Result<Self, git2::Error> {
        let mut cadence = CadenceMetric {
            global_commits_per_day: Self::global_commits_per_day(repo)?,
            author_commits_per_day: HashMap::new(),
            selected_index: 0,
        };
        for author in repo.get_authors()? {
            let commit_dates: Vec<DateTime<Utc>> = repo
                .get_author_commits(&author)?
                .filter_map(|commit| DateTime::from_timestamp_secs(commit.time().seconds()))
                .collect();

            cadence
                .author_commits_per_day
                .insert(author, commits_per_day(&commit_dates));
        }
        Ok(cadence)
    }

    pub fn next_index(&mut self) {
        if !self.author_commits_per_day.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.author_commits_per_day.len()
        }
    }

    pub fn previous_index(&mut self) {
        if !self.author_commits_per_day.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.author_commits_per_day.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }
}

fn commits_per_day(commits: &[DateTime<Utc>]) -> f32 {
    match telescope_time(&commits) {
        Some(delta) => {
            let seconds_avg = delta.as_seconds_f32();
            if seconds_avg > 0.0 {
                (1.0 / seconds_avg) * 60.0 * 60.0 * 24.0
            } else {
                0.0
            }
        }
        None => 0.0,
    }
}

//https://en.wikipedia.org/wiki/Telescoping_series
fn telescope_time(datetimes: &[DateTime<Utc>]) -> Option<TimeDelta> {
    if datetimes.len() < 2 {
        return None;
    }

    // the middle dates all cancel when summing over their differences as pairs
    // and we are left with the first and last only
    let total_duration = *datetimes.first()? - *datetimes.last()?;
    let count = (datetimes.len() - 1) as i32;

    total_duration.checked_div(count)
}

use super::RenderMetric;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Borders, List, ListItem, ListState, Padding},
};

impl RenderMetric for CadenceMetric {
    fn update(&mut self, key: crossterm::event::KeyCode) {
        match key {
            KeyCode::Down | KeyCode::Char('j') => self.next_index(),
            KeyCode::Up | KeyCode::Char('k') => self.previous_index(),
            _ => {}
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect, state: &State) {
        let block_widg = Block::default()
            .title("Cadence")
            .title_alignment(ratatui::layout::HorizontalAlignment::Center)
            .borders(Borders::ALL)
            .padding(Padding::horizontal(1));

        frame.render_widget(block_widg.clone(), area);

        let inner_area = block_widg.inner(area);

        if let Some(cadence) = &state.cadence {
            let left_constraint = Constraint::Percentage(60);
            let right_constraint = Constraint::Percentage(40);
            let middle_spacer = Constraint::Percentage(2);

            let main_columns =
                Layout::horizontal([left_constraint, middle_spacer, right_constraint])
                    .split(inner_area);

            let left_column = main_columns[0];
            let right_column = main_columns[2];

            let list_items: Vec<ListItem> = cadence
                .author_commits_per_day
                .iter()
                .map(|item| {
                    let line = ratatui::text::Line::from(vec![
                        ratatui::text::Span::styled(
                            format!("{:<15}", item.0),
                            ratatui::style::Style::default()
                                .add_modifier(ratatui::style::Modifier::BOLD),
                        ),
                        ratatui::text::Span::styled(
                            format!(" {} commits per day", item.1),
                            ratatui::style::Style::default().fg(ratatui::style::Color::Cyan),
                        ),
                    ]);
                    ListItem::new(line)
                })
                .collect();

            let list_widget = List::new(list_items)
                .highlight_style(
                    ratatui::style::Style::default()
                        .bg(ratatui::style::Color::Indexed(237))
                        .add_modifier(ratatui::style::Modifier::BOLD),
                )
                .highlight_symbol("> ");

            let mut list_state = ListState::default().with_selected(Some(cadence.selected_index));

            frame.render_stateful_widget(list_widget, left_column, &mut list_state);

            // Render placeholder on the right
            util::draw_placeholder(
                frame,
                right_column,
                "placeholder",
                ratatui::style::Color::Green,
            );
        } else {
            // err
        }
    }
}

// fn chart(&self, frame: &mut Frame, area: Rect, state: &State) {

// }

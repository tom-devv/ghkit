
use chrono::{DateTime, TimeDelta, Utc};
use crossterm::event::KeyCode;

use crate::{
    git::kit::GRepo,
    tui::{state::State, util::draw_placeholder},
};
#[derive(Debug)]
pub struct CadenceMetric {
    pub global_commits_per_day: u32,
    pub author_commits_per_day: Vec<AuthorCommits>,
    pub selected_index: usize,
    pub selected_author: Option<AuthorCommits>,
}

#[derive(Debug, Clone)]

pub struct AuthorCommits {
    pub name: String,
    pub commit_per_day: u32,
}

impl CadenceMetric {
    pub fn author_commits_per_day(repo: &GRepo, email: &str) -> Result<u32, git2::Error> {
        let commit_dates: Vec<DateTime<Utc>> = repo
            .get_author_commits(email)?
            .filter_map(|commit| DateTime::from_timestamp_secs(commit.time().seconds()))
            .collect();

        Ok(commits_per_day(&commit_dates))
    }

    pub fn global_commits_per_day(repo: &GRepo) -> Result<u32, git2::Error> {
        let commit_dates: Vec<DateTime<Utc>> = repo
            .iter_commits()?
            .filter_map(|commit| DateTime::from_timestamp_secs(commit.time().seconds()))
            .collect();

        Ok(commits_per_day(&commit_dates))
    }

    pub fn full_report(repo: &GRepo) -> Result<Self, git2::Error> {
        let mut cadence = CadenceMetric {
            global_commits_per_day: Self::global_commits_per_day(repo)?,
            author_commits_per_day: Vec::new(),
            selected_index: 0,
            selected_author: None,
        };
        for author in repo.get_authors()? {
            let commit_dates: Vec<DateTime<Utc>> = repo
                .get_author_commits(&author)?
                .filter_map(|commit| DateTime::from_timestamp_secs(commit.time().seconds()))
                .collect();

            cadence.author_commits_per_day.push(AuthorCommits {
                name: author,
                commit_per_day: commits_per_day(&commit_dates),
            });
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

    pub fn select(&mut self) {
        if self.selected_author.is_some() {
            self.selected_author = None
        } else {
            self.selected_author = Some(self.author_commits_per_day[self.selected_index].clone());
        }
    }

    pub fn more_info(&self, frame: &mut Frame) {
        let area = frame
            .area()
            .centered(Constraint::Max(10), Constraint::Max(10));
        draw_placeholder(frame, area, "xyz", ratatui::style::Color::Green);
    }
}

fn commits_per_day(commits: &[DateTime<Utc>]) -> u32 {
    match telescope_time(&commits) {
        Some(delta) => {
            let seconds_avg = delta.as_seconds_f32();
            if seconds_avg > 0.0 {
                ((1.0 / seconds_avg) * 60.0 * 60.0 * 24.0) as u32
            } else {
                0.0 as u32
            }
        }
        None => 0.0 as u32,
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
    style::Style,
    widgets::{BarChart, Block, Borders, List, ListItem, ListState, Padding},
};

impl RenderMetric for CadenceMetric {
    fn update(&mut self, key: crossterm::event::KeyCode) {
        match key {
            KeyCode::Down | KeyCode::Char('j') => self.next_index(),
            KeyCode::Up | KeyCode::Char('k') => self.previous_index(),
            KeyCode::Enter => self.select(),
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

        // Show more info frame
        if self.selected_author.is_some() {
            self.more_info(frame);
        }

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
                            format!("{:<15}", item.name),
                            ratatui::style::Style::default()
                                .add_modifier(ratatui::style::Modifier::BOLD),
                        ),
                        ratatui::text::Span::styled(
                            format!(" {} commits per day", item.commit_per_day),
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

            let mut authors: Vec<(&String, &u32)> = cadence
                .author_commits_per_day
                .iter()
                .map(|ac| (&ac.name, &ac.commit_per_day))
                .collect();
            authors.sort_by(|a, b| a.1.cmp(b.1));

            let chart_data: Vec<(&str, u64)> = authors
                .into_iter()
                .map(|(author, commits)| (author.as_str(), ((*commits) as f32).round() as u64))
                .collect();

            let barchar = BarChart::default()
                .block(Block::bordered().title("BarChart"))
                .bar_width(3)
                .bar_gap(5)
                .group_gap(3)
                .bar_style(Style::new().white())
                .value_style(Style::new().red().bold())
                .label_style(Style::new().white())
                .data(&chart_data)
                .max(4);

            frame.render_widget(barchar, right_column);
        } else {
            // err
        }
    }
}

// fn chart(&self, frame: &mut Frame, area: Rect, state: &State) {

// }

use std::ops::Index;

use chrono::{DateTime, TimeDelta, Utc};
use crossterm::event::{KeyCode, KeyEvent};

use crate::{git::kit::GRepo, tui::ui::draw_placeholder};

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{BarChart, Block, BorderType, Borders, Cell, Padding, Row, Table, TableState},
};

#[derive(Debug, Clone)]
pub struct CadenceData {
    pub global_commits_per_week: u32,
    pub author_commits_per_week: Vec<AuthorCommits>,
}

#[derive(Debug)]
pub struct CadencePage {
    pub data: CadenceData,
    pub selected_index: usize,
    pub selected_author: Option<AuthorCommits>,
    pub table_state: TableState,
}

#[derive(Debug, Clone)]

pub struct AuthorCommits {
    pub name: String,
    pub commits_per_week: u32,
}

impl CadencePage {
    pub fn new(data: CadenceData) -> Self {
        Self {
            data,
            selected_index: 0,
            selected_author: None,
            table_state: TableState::default().with_selected(Some(0)),
        }
    }

    pub fn handle_key(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Down | KeyCode::Char('j') => self.next_index(),
            KeyCode::Up | KeyCode::Char('k') => self.previous_index(),
            KeyCode::Enter => self.select(),
            _ => {}
        };
    }

    pub fn next_index(&mut self) {
        if !self.data.author_commits_per_week.is_empty() {
            self.selected_index =
                (self.selected_index + 1) % self.data.author_commits_per_week.len();
            self.table_state.select(Some(self.selected_index));
        }
    }

    pub fn previous_index(&mut self) {
        if !self.data.author_commits_per_week.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.data.author_commits_per_week.len() - 1;
            } else {
                self.selected_index -= 1;
            }

            self.table_state.select(Some(self.selected_index));
        }
    }

    pub fn select(&mut self) {
        if self.selected_author.is_some() {
            self.selected_author = None
        } else {
            self.selected_author =
                Some(self.data.author_commits_per_week[self.selected_index].clone());
        }
    }

    pub fn more_info(&self, frame: &mut Frame) {
        let area = frame
            .area()
            .centered(Constraint::Percentage(50), Constraint::Percentage(50));

        let author = self.data.author_commits_per_week.index(self.selected_index);
        draw_placeholder(
            frame,
            area,
            format!("Author: {}", author.name).as_str(),
            ratatui::style::Color::Green,
        );
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .padding(Padding::horizontal(1));

        frame.render_widget(block.clone(), area);

        let inner_area = block.inner(area);

        let left_constraint = Constraint::Percentage(60);
        let right_constraint = Constraint::Percentage(40);
        let middle_spacer = Constraint::Percentage(2);

        let main_columns = Layout::horizontal([left_constraint, middle_spacer, right_constraint])
            .split(inner_area);

        let left_column = main_columns[0];
        let right_column = main_columns[2];

        frame.render_stateful_widget(self.author_table(), left_column, &mut self.table_state);

        frame.render_widget(self.chart(), right_column);

        // Show more info frame last, this will draw it on top
        if self.selected_author.is_some() {
            self.more_info(frame);
        }
    }

    fn chart(&self) -> BarChart<'_> {
        let mut authors: Vec<(&String, &u32)> = self
            .data
            .author_commits_per_week
            .iter()
            .map(|ac| (&ac.name, &ac.commits_per_week))
            .collect();
        authors.sort_by(|a, b| a.1.cmp(b.1));

        let chart_data: Vec<(&str, u64)> = authors
            .into_iter()
            .map(|(author, commits)| (author.as_str(), ((*commits) as f32).round() as u64))
            .collect();

        BarChart::default()
            .block(
                Block::default()
                    .title(" Activity Overview ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::HeavyDoubleDashed),
            )
            .data(&chart_data)
            .bar_width(5)
            .bar_gap(2)
            .bar_style(Style::default().fg(Color::Cyan))
            .value_style(Style::default().fg(Color::Black).bg(Color::Cyan))
    }

    fn author_table(&self) -> Table<'static> {
        let widths = [Constraint::Percentage(50), Constraint::Percentage(30)];

        let rows: Vec<Row> = self
            .data
            .author_commits_per_week
            .iter()
            .map(|item| {
                Row::new(vec![
                    Cell::from(item.name.clone())
                        .style(Style::default().add_modifier(Modifier::BOLD)),
                    Cell::from(format!("{:.2} / week", item.commits_per_week))
                        .style(Style::default().fg(Color::DarkGray)),
                ])
            })
            .collect();

        Table::new(rows, widths)
            .row_highlight_style(Style::default().bg(Color::Indexed(237)))
            .highlight_symbol("> ")
    }
}

impl CadenceData {
    pub fn author_commits_per_week(repo: &GRepo, email: &str) -> Result<u32, git2::Error> {
        let commit_dates: Vec<DateTime<Utc>> = repo
            .get_author_commits(email)?
            .filter_map(|commit| DateTime::from_timestamp_secs(commit.time().seconds()))
            .collect();

        Ok(commits_per_week(&commit_dates))
    }

    pub fn global_commits_per_week(repo: &GRepo) -> Result<u32, git2::Error> {
        let commit_dates: Vec<DateTime<Utc>> = repo
            .iter_commits()?
            .filter_map(|commit| DateTime::from_timestamp_secs(commit.time().seconds()))
            .collect();

        Ok(commits_per_week(&commit_dates))
    }

    pub fn full_report(repo: &GRepo) -> Result<Self, git2::Error> {
        let mut cadence = CadenceData {
            global_commits_per_week: Self::global_commits_per_week(repo)?,
            author_commits_per_week: Vec::new(),
        };
        for author in repo.get_authors()? {
            let commit_dates: Vec<DateTime<Utc>> = repo
                .get_author_commits(&author)?
                .filter_map(|commit| DateTime::from_timestamp_secs(commit.time().seconds()))
                .collect();

            cadence.author_commits_per_week.push(AuthorCommits {
                name: author,
                commits_per_week: commits_per_week(&commit_dates),
            });
        }
        Ok(cadence)
    }
}

fn commits_per_week(commits: &[DateTime<Utc>]) -> u32 {
    match telescope_time(&commits) {
        Some(delta) => {
            let seconds_avg = delta.as_seconds_f32();
            if seconds_avg > 0.0 {
                ((1.0 / seconds_avg) * 60.0 * 60.0 * 24.0 * 7.0) as u32
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

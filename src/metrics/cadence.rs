use std::{
    collections::{self, HashMap},
    iter::Map,
};

use chrono::{DateTime, TimeDelta, Utc};
use git2::ConfigLevel::Global;

use crate::git::kit::GRepo;
#[derive(Debug)]
pub struct CadenceMetric {
    pub global_commits_per_day: f32,
    pub author_commits_per_day: HashMap<String, f32>,
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

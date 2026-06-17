use chrono::{Datelike, Duration, Local, NaiveDate, Timelike};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::models::{AuthorKey, CommitRecord, RepoInfo, ScanResult};

const STALE_DAYS: i64 = 30;
const STOPWORDS: &[&str] = &[
    "the", "a", "an", "and", "or", "to", "of", "in", "for", "on", "with", "is", "it", "this", "that",
    "from", "at", "by", "as", "be", "was", "were", "are", "fix", "add", "update", "remove",
];

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthorStats {
    pub author: AuthorKey,
    pub commit_count: usize,
    pub total_message_length: usize,
    pub repos: HashSet<String>,
}

impl AuthorStats {
    pub fn avg_message_length(&self) -> f64 {
        if self.commit_count == 0 {
            0.0
        } else {
            self.total_message_length as f64 / self.commit_count as f64
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RepoStats {
    pub repo_name: String,
    pub repo_path: String,
    pub commit_count: usize,
    pub author_count: usize,
    pub last_commit_at: Option<chrono::DateTime<Local>>,
    pub is_stale: bool,
    pub remote_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalStats {
    pub total_repos: usize,
    pub total_commits: usize,
    pub total_authors: usize,
    pub stale_repos: usize,
    pub avg_message_length: f64,
    pub commits_by_hour: [usize; 24],
    pub commits_by_weekday: [usize; 7],
    pub commits_by_month: [usize; 12],
    pub peak_hour: u8,
    pub peak_weekday: String,
    pub velocity_7d: usize,
    pub velocity_14d: usize,
    pub velocity_30d: usize,
    pub authors: Vec<AuthorStats>,
    pub repos: Vec<RepoStats>,
    pub top_words: Vec<(String, usize)>,
    pub daily_commits: Vec<(NaiveDate, usize)>,
    pub heatmap: HeatmapData,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HeatmapData {
    pub weeks: usize,
    pub cells: Vec<(NaiveDate, u8)>,
    pub max_level: u8,
}

pub fn compute_stats(scan: &ScanResult) -> GlobalStats {
    let mut stats = GlobalStats {
        total_repos: scan.repos.len(),
        total_commits: scan.commits.len(),
        ..Default::default()
    };

    let mut author_map: HashMap<AuthorKey, AuthorStats> = HashMap::new();
    let mut repo_commit_counts: HashMap<String, usize> = HashMap::new();
    let mut repo_authors: HashMap<String, HashSet<AuthorKey>> = HashMap::new();
    let mut word_counts: HashMap<String, usize> = HashMap::new();
    let mut daily: HashMap<NaiveDate, usize> = HashMap::new();
    let mut total_message_length = 0usize;

    let now = Local::now();
    let day_7 = now - Duration::days(7);
    let day_14 = now - Duration::days(14);
    let day_30 = now - Duration::days(30);

    for commit in &scan.commits {
        total_message_length += commit.message_length;

        let hour = commit.timestamp.hour() as usize;
        stats.commits_by_hour[hour] += 1;

        let weekday = commit.timestamp.weekday().num_days_from_monday() as usize;
        stats.commits_by_weekday[weekday] += 1;

        let month = (commit.timestamp.month() - 1) as usize;
        stats.commits_by_month[month] += 1;

        let date = commit.timestamp.date_naive();
        *daily.entry(date).or_default() += 1;

        if commit.timestamp >= day_7 {
            stats.velocity_7d += 1;
        }
        if commit.timestamp >= day_14 {
            stats.velocity_14d += 1;
        }
        if commit.timestamp >= day_30 {
            stats.velocity_30d += 1;
        }

        let entry = author_map.entry(commit.author.clone()).or_insert_with(|| AuthorStats {
            author: commit.author.clone(),
            ..Default::default()
        });
        entry.commit_count += 1;
        entry.total_message_length += commit.message_length;
        entry.repos.insert(commit.repo_name.clone());

        *repo_commit_counts.entry(commit.repo_name.clone()).or_default() += 1;
        repo_authors
            .entry(commit.repo_name.clone())
            .or_default()
            .insert(commit.author.clone());

        tokenize_message(&commit.message, &mut word_counts);
    }

    stats.total_authors = author_map.len();
    stats.avg_message_length = if stats.total_commits == 0 {
        0.0
    } else {
        total_message_length as f64 / stats.total_commits as f64
    };

    stats.peak_hour = stats
        .commits_by_hour
        .iter()
        .enumerate()
        .max_by_key(|(_, c)| *c)
        .map(|(h, _)| h as u8)
        .unwrap_or(0);

    stats.peak_weekday = stats
        .commits_by_weekday
        .iter()
        .enumerate()
        .max_by_key(|(_, c)| *c)
        .map(|(d, _)| weekday_name(d))
        .unwrap_or_else(|| "Mon".into());

    stats.authors = author_map.into_values().collect();
    stats.authors.sort_by(|a, b| b.commit_count.cmp(&a.commit_count));

    stats.repos = scan
        .repos
        .iter()
        .map(|repo| build_repo_stats(repo, &repo_commit_counts, &repo_authors))
        .collect();
    stats.repos.sort_by(|a, b| b.commit_count.cmp(&a.commit_count));

    stats.stale_repos = stats.repos.iter().filter(|r| r.is_stale).count();

    stats.top_words = top_words(&word_counts, 15);
    stats.daily_commits = daily_commits_series(&daily, 90);
    stats.heatmap = build_heatmap(&daily);

    stats
}

fn build_repo_stats(
    repo: &RepoInfo,
    commit_counts: &HashMap<String, usize>,
    repo_authors: &HashMap<String, HashSet<AuthorKey>>,
) -> RepoStats {
    let is_stale = repo
        .last_commit_at
        .map(|t| Local::now().signed_duration_since(t).num_days() > STALE_DAYS)
        .unwrap_or(true);

    RepoStats {
        repo_name: repo.name.clone(),
        repo_path: repo.path.display().to_string(),
        commit_count: commit_counts.get(&repo.name).copied().unwrap_or(0),
        author_count: repo_authors.get(&repo.name).map(|a| a.len()).unwrap_or(0),
        last_commit_at: repo.last_commit_at,
        is_stale,
        remote_url: repo.remote_url.clone(),
    }
}

fn tokenize_message(message: &str, counts: &mut HashMap<String, usize>) {
    for word in message
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 2)
    {
        if STOPWORDS.contains(&word) {
            continue;
        }
        *counts.entry(word.to_string()).or_default() += 1;
    }
}

fn top_words(counts: &HashMap<String, usize>, limit: usize) -> Vec<(String, usize)> {
    let mut words: Vec<(String, usize)> = counts.iter().map(|(k, v)| (k.clone(), *v)).collect();
    words.sort_by(|a, b| b.1.cmp(&a.1));
    words.truncate(limit);
    words
}

fn daily_commits_series(daily: &HashMap<NaiveDate, usize>, days: i64) -> Vec<(NaiveDate, usize)> {
    let today = Local::now().date_naive();
    let start = today - Duration::days(days - 1);
    let mut series = Vec::new();
    let mut current = start;
    while current <= today {
        series.push((current, daily.get(&current).copied().unwrap_or(0)));
        current += Duration::days(1);
    }
    series
}

fn build_heatmap(daily: &HashMap<NaiveDate, usize>) -> HeatmapData {
    let today = Local::now().date_naive();
    let start = today - Duration::days(364);
    let max_count = daily
        .iter()
        .filter(|(d, _)| **d >= start && **d <= today)
        .map(|(_, c)| *c)
        .max()
        .unwrap_or(0);

    let mut cells = Vec::new();
    let mut current = start;
    while current <= today {
        let count = daily.get(&current).copied().unwrap_or(0);
        let level = heatmap_level(count, max_count);
        cells.push((current, level));
        current += Duration::days(1);
    }

    let weeks = ((today - start).num_days() as usize / 7) + 1;

    HeatmapData {
        weeks,
        cells,
        max_level: 3,
    }
}

pub fn heatmap_level(count: usize, max_count: usize) -> u8 {
    if count == 0 {
        0
    } else if max_count == 0 {
        1
    } else {
        let ratio = count as f64 / max_count as f64;
        if ratio < 0.25 {
            1
        } else if ratio < 0.6 {
            2
        } else {
            3
        }
    }
}

fn weekday_name(index: usize) -> String {
    match index {
        0 => "Mon",
        1 => "Tue",
        2 => "Wed",
        3 => "Thu",
        4 => "Fri",
        5 => "Sat",
        _ => "Sun",
    }
    .into()
}

pub fn night_owl_ratio(_stats: &GlobalStats, commits: &[CommitRecord]) -> f64 {
    if commits.is_empty() {
        return 0.0;
    }
    let night = commits
        .iter()
        .filter(|c| c.timestamp.hour() < 5)
        .count();
    night as f64 / commits.len() as f64
}

pub fn early_bird_ratio(commits: &[CommitRecord]) -> f64 {
    if commits.is_empty() {
        return 0.0;
    }
    let early = commits
        .iter()
        .filter(|c| c.timestamp.hour() < 8)
        .count();
    early as f64 / commits.len() as f64
}

pub fn active_days(commits: &[CommitRecord]) -> usize {
    let days: HashSet<NaiveDate> = commits.iter().map(|c| c.timestamp.date_naive()).collect();
    days.len()
}

pub fn lone_wolf_repos(repos: &[RepoStats]) -> usize {
    repos.iter().filter(|r| r.author_count == 1 && r.commit_count > 0).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AuthorKey, CommitRecord, RepoInfo, ScanResult};
    use chrono::Local;
    use std::path::PathBuf;

    fn sample_commit(hour: u32, msg: &str) -> CommitRecord {
        let mut ts = Local::now();
        ts = ts
            .with_hour(hour)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap();
        CommitRecord {
            repo_path: PathBuf::from("/tmp/r"),
            repo_name: "r".into(),
            oid: "abc".into(),
            parents: vec![],
            author: AuthorKey {
                name: "A".into(),
                email: "a@x.com".into(),
            },
            timestamp: ts,
            message: msg.into(),
            message_length: msg.len(),
            files_changed: 1,
            insertions: 1,
            deletions: 0,
            branch: Some("main".into()),
        }
    }

    #[test]
    fn detects_stale_repo() {
        let old = Local::now() - Duration::days(40);
        let scan = ScanResult {
            repos: vec![RepoInfo {
                path: PathBuf::from("/tmp/r"),
                name: "r".into(),
                remote_url: None,
                head_oid: None,
                default_branch: None,
                commit_count: 1,
                last_commit_at: Some(old),
                authors: vec![],
            }],
            commits: vec![],
            scan_paths: vec![],
            scanned_at: Local::now(),
        };
        let stats = compute_stats(&scan);
        assert_eq!(stats.stale_repos, 1);
    }

    #[test]
    fn heatmap_levels() {
        assert_eq!(heatmap_level(0, 10), 0);
        assert_eq!(heatmap_level(2, 10), 1);
        assert_eq!(heatmap_level(5, 10), 2);
        assert_eq!(heatmap_level(9, 10), 3);
    }

    #[test]
    fn velocity_windows() {
        let scan = ScanResult {
            repos: vec![],
            commits: vec![sample_commit(10, "fix bug in parser")],
            scan_paths: vec![],
            scanned_at: Local::now(),
        };
        let stats = compute_stats(&scan);
        assert_eq!(stats.velocity_7d, 1);
        assert!(!stats.top_words.is_empty());
    }
}

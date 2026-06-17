use serde::{Deserialize, Serialize};

use crate::stats::{active_days, early_bird_ratio, lone_wolf_repos, night_owl_ratio, GlobalStats};
use crate::models::ScanResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub unlocked: bool,
}

pub fn evaluate_achievements(scan: &ScanResult, stats: &GlobalStats) -> Vec<Achievement> {
    vec![
        night_owl(scan, stats),
        early_bird(scan),
        marathon(scan),
        lone_wolf(stats),
        commitizen(stats),
    ]
}

fn night_owl(scan: &ScanResult, stats: &GlobalStats) -> Achievement {
    let ratio = night_owl_ratio(stats, &scan.commits);
    Achievement {
        id: "night_owl".into(),
        title: "Night Owl".into(),
        description: "50%+ commits between midnight and 5 AM".into(),
        unlocked: ratio >= 0.5,
    }
}

fn early_bird(scan: &ScanResult) -> Achievement {
    let ratio = early_bird_ratio(&scan.commits);
    Achievement {
        id: "early_bird".into(),
        title: "Early Bird".into(),
        description: "50%+ commits before 8 AM".into(),
        unlocked: ratio >= 0.5,
    }
}

fn marathon(scan: &ScanResult) -> Achievement {
    let days = active_days(&scan.commits);
    Achievement {
        id: "marathon".into(),
        title: "Marathon".into(),
        description: "Commits on 30+ distinct days".into(),
        unlocked: days >= 30,
    }
}

fn lone_wolf(stats: &GlobalStats) -> Achievement {
    let count = lone_wolf_repos(&stats.repos);
    Achievement {
        id: "lone_wolf".into(),
        title: "Lone Wolf".into(),
        description: "Sole contributor in at least one repo".into(),
        unlocked: count > 0,
    }
}

fn commitizen(stats: &GlobalStats) -> Achievement {
    Achievement {
        id: "commitizen".into(),
        title: "Commitizen".into(),
        description: "Average commit message length over 50 characters".into(),
        unlocked: stats.avg_message_length > 50.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::RepoStats;
    use chrono::Local;

    #[test]
    fn commitizen_unlocked() {
        let stats = GlobalStats {
            avg_message_length: 55.0,
            ..Default::default()
        };
        let scan = ScanResult {
            repos: vec![],
            commits: vec![],
            scan_paths: vec![],
            scanned_at: Local::now(),
        };
        let achievements = evaluate_achievements(&scan, &stats);
        let commitizen = achievements.iter().find(|a| a.id == "commitizen").unwrap();
        assert!(commitizen.unlocked);
    }

    #[test]
    fn lone_wolf_unlocked() {
        let stats = GlobalStats {
            repos: vec![RepoStats {
                repo_name: "solo".into(),
                repo_path: "/solo".into(),
                commit_count: 5,
                author_count: 1,
                last_commit_at: None,
                is_stale: false,
                remote_url: None,
            }],
            ..Default::default()
        };
        let scan = ScanResult {
            repos: vec![],
            commits: vec![],
            scan_paths: vec![],
            scanned_at: Local::now(),
        };
        let achievements = evaluate_achievements(&scan, &stats);
        let lone = achievements.iter().find(|a| a.id == "lone_wolf").unwrap();
        assert!(lone.unlocked);
    }
}

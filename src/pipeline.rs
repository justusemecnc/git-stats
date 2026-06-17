use anyhow::Result;
use chrono::Local;

use crate::achievements::Achievement;
use crate::cache::CacheStore;
use crate::config::Config;
use crate::git::extract::extract_all;
use crate::models::ScanResult;
use crate::scanner::scan_repositories;
use crate::stats::{compute_stats, GlobalStats};

#[derive(Debug, Clone)]
pub struct PipelineOutput {
    pub scan: ScanResult,
    pub stats: GlobalStats,
    pub achievements: Vec<Achievement>,
}

pub fn run_scan(config: &Config) -> Result<PipelineOutput> {
    let repos = scan_repositories(config)?;
    let (commits, updated_repos) = extract_all(&repos, config)?;
    let final_repos = merge_repo_info(repos, updated_repos);

    if config.cache.enabled {
        let mut cache = CacheStore::load().unwrap_or_default();
        for repo in &final_repos {
            cache.upsert(
                repo.path.clone(),
                repo.remote_url.clone(),
                repo.head_oid.clone(),
            );
        }
        let _ = cache.save();
    }

    let scan = ScanResult {
        repos: final_repos,
        commits,
        scan_paths: config.scan_paths.clone(),
        scanned_at: Local::now(),
    };

    let stats = compute_stats(&scan);
    let achievements = crate::achievements::evaluate_achievements(&scan, &stats);

    Ok(PipelineOutput {
        scan,
        stats,
        achievements,
    })
}

fn merge_repo_info(
    mut base: Vec<crate::models::RepoInfo>,
    updated: Vec<crate::models::RepoInfo>,
) -> Vec<crate::models::RepoInfo> {
    for u in updated {
        if let Some(r) = base.iter_mut().find(|r| r.path == u.path) {
            *r = u;
        }
    }
    base
}

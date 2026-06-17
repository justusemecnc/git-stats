use anyhow::{Context, Result};
use chrono::{Local, TimeZone};
use git2::{DiffOptions, Repository};
use rayon::prelude::*;
use std::collections::HashSet;

use crate::config::Config;
use crate::models::{AuthorKey, CommitRecord, RepoInfo};

pub fn extract_repo_commits(repo_info: &RepoInfo, config: &Config) -> Result<(Vec<CommitRecord>, RepoInfo)> {
    let repo = Repository::open(&repo_info.path)
        .with_context(|| format!("opening repo at {}", repo_info.path.display()))?;

    let mut revwalk = repo.revwalk().context("creating revwalk")?;
    revwalk.push_head().context("pushing HEAD")?;
    revwalk.set_sorting(git2::Sort::TIME).context("setting sort")?;

    let default_branch = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(|s| s.to_string()));

    let mut commits = Vec::new();
    let mut authors = HashSet::new();
    let mut last_commit_at = None;
    let max = config.max_commits_per_repo.unwrap_or(usize::MAX);

    for (i, oid) in revwalk.enumerate() {
        if i >= max {
            break;
        }
        let oid = oid.context("walking commits")?;
        let commit = repo.find_commit(oid).context("finding commit")?;

        let timestamp = Local
            .timestamp_opt(commit.time().seconds(), 0)
            .single()
            .context("invalid commit timestamp")?;

        if last_commit_at.is_none() {
            last_commit_at = Some(timestamp);
        }

        let author = commit.author();
        let author_key = AuthorKey {
            name: author.name().unwrap_or("unknown").to_string(),
            email: author.email().unwrap_or("unknown@local").to_string(),
        };
        authors.insert(author_key.clone());

        let message = commit.message().unwrap_or("").to_string();
        let message_length = message.len();

        let parents: Vec<String> = commit.parent_ids().map(|p| p.to_string()).collect();

        let (files_changed, insertions, deletions) = commit_diff_stats(&repo, &commit)?;

        commits.push(CommitRecord {
            repo_path: repo_info.path.clone(),
            repo_name: repo_info.name.clone(),
            oid: oid.to_string(),
            parents,
            author: author_key,
            timestamp,
            message,
            message_length,
            files_changed,
            insertions,
            deletions,
            branch: default_branch.clone(),
        });
    }

    let mut updated = repo_info.clone();
    updated.commit_count = commits.len();
    updated.last_commit_at = last_commit_at;
    updated.authors = authors.into_iter().collect();
    updated.default_branch = default_branch;

    Ok((commits, updated))
}

fn commit_diff_stats(repo: &Repository, commit: &git2::Commit) -> Result<(usize, usize, usize)> {
    let tree = commit.tree().ok();
    let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());

    let mut diff_opts = DiffOptions::new();

    let diff = match (tree, parent_tree) {
        (Some(t), Some(pt)) => repo.diff_tree_to_tree(Some(&pt), Some(&t), Some(&mut diff_opts))?,
        (Some(t), None) => repo.diff_tree_to_tree(None, Some(&t), Some(&mut diff_opts))?,
        _ => return Ok((0, 0, 0)),
    };

    let stats = diff.stats()?;
    Ok((
        stats.files_changed(),
        stats.insertions() as usize,
        stats.deletions() as usize,
    ))
}

pub fn extract_all(repos: &[RepoInfo], config: &Config) -> Result<(Vec<CommitRecord>, Vec<RepoInfo>)> {
    if config.parallel {
        let results: Vec<Result<(Vec<CommitRecord>, RepoInfo)>> = repos
            .par_iter()
            .map(|repo| extract_repo_commits(repo, config))
            .collect();
        merge_results(results)
    } else {
        let results: Vec<Result<(Vec<CommitRecord>, RepoInfo)>> = repos
            .iter()
            .map(|repo| extract_repo_commits(repo, config))
            .collect();
        merge_results(results)
    }
}

fn merge_results(results: Vec<Result<(Vec<CommitRecord>, RepoInfo)>>) -> Result<(Vec<CommitRecord>, Vec<RepoInfo>)> {
    let mut all_commits = Vec::new();
    let mut updated_repos = Vec::new();
    for result in results {
        match result {
            Ok((commits, repo)) => {
                all_commits.extend(commits);
                updated_repos.push(repo);
            }
            Err(e) => {
                eprintln!("warning: {e:#}");
            }
        }
    }
    Ok((all_commits, updated_repos))
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::Repository;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    fn make_repo_with_commits(path: &Path, count: usize) {
        let repo = Repository::init(path).unwrap();
        let sig = git2::Signature::now("Alice", "alice@example.com").unwrap();
        let mut parent: Option<git2::Oid> = None;

        for i in 0..count {
            let mut index = repo.index().unwrap();
            let blob = repo.blob(format!("content {i}").as_bytes()).unwrap();
            let _ = index.add(&git2::IndexEntry {
                ctime: git2::IndexTime::new(0, 0),
                mtime: git2::IndexTime::new(0, 0),
                dev: 0,
                ino: 0,
                mode: 0o100644,
                uid: 0,
                gid: 0,
                file_size: 0,
                id: blob,
                flags: 0,
                flags_extended: 0,
                path: format!("file{i}.txt").into(),
            });
            index.write().unwrap();
            let tree_id = index.write_tree().unwrap();
            let tree = repo.find_tree(tree_id).unwrap();

            let oid = if let Some(p) = parent {
                let parent_commit = repo.find_commit(p).unwrap();
                repo.commit(
                    Some("HEAD"),
                    &sig,
                    &sig,
                    &format!("commit {i}"),
                    &tree,
                    &[&parent_commit],
                )
                .unwrap()
            } else {
                repo.commit(Some("HEAD"), &sig, &sig, &format!("commit {i}"), &tree, &[])
                    .unwrap()
            };
            parent = Some(oid);
        }
    }

    #[test]
    fn extracts_commit_metadata() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("repo");
        fs::create_dir_all(&path).unwrap();
        make_repo_with_commits(&path, 3);

        let repo_info = RepoInfo {
            path: path.clone(),
            name: "repo".into(),
            remote_url: None,
            head_oid: None,
            default_branch: None,
            commit_count: 0,
            last_commit_at: None,
            authors: vec![],
        };

        let config = Config::default();
        let (commits, updated) = extract_repo_commits(&repo_info, &config).unwrap();
        assert_eq!(commits.len(), 3);
        assert_eq!(updated.commit_count, 3);
        assert_eq!(commits[0].author.email, "alice@example.com");
    }
}

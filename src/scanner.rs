use anyhow::{Context, Result};
use git2::Repository;
use rayon::prelude::*;
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::config::Config;
use crate::models::RepoInfo;

#[derive(Debug, Clone)]
pub struct DiscoveredRepo {
    pub path: PathBuf,
    pub git_dir: PathBuf,
}

pub fn discover_repos(config: &Config) -> Result<Vec<DiscoveredRepo>> {
    let mut found = Vec::new();
    let mut seen_git_dirs = std::collections::HashSet::new();

    for scan_path in &config.scan_paths {
        if !scan_path.exists() {
            continue;
        }
        let mut walker = WalkDir::new(scan_path).follow_links(false);
        if let Some(depth) = config.max_depth {
            walker = walker.max_depth(depth);
        }

        for entry in walker
            .into_iter()
            .filter_entry(|e| should_visit(e, config))
            .filter_map(|e| e.ok())
        {
            let file_name = entry.file_name().to_string_lossy();
            if file_name == ".git" {
                let git_dir = entry.path().to_path_buf();
                let key = git_dir.to_string_lossy().into_owned();
                if seen_git_dirs.insert(key) {
                    let repo_path = entry
                        .path()
                        .parent()
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|| scan_path.clone());
                    found.push(DiscoveredRepo {
                        path: repo_path,
                        git_dir,
                    });
                }
            }
        }
    }

    Ok(found)
}

fn should_visit(entry: &walkdir::DirEntry, config: &Config) -> bool {
    let name = entry.file_name().to_string_lossy();
    if name == ".git" {
        return true;
    }
    if is_inside_git_dir(entry.path()) {
        return false;
    }
    if entry.file_type().is_dir() && config.is_excluded(&name) {
        return false;
    }
    true
}

fn is_inside_git_dir(path: &std::path::Path) -> bool {
    path.parent()
        .into_iter()
        .flat_map(|p| p.ancestors())
        .any(|p| p.file_name().is_some_and(|n| n == ".git"))
}

pub fn validate_repos(repos: Vec<DiscoveredRepo>, parallel: bool) -> Vec<RepoInfo> {
    if parallel {
        repos
            .into_par_iter()
            .filter_map(|repo| validate_repo(&repo).ok())
            .collect()
    } else {
        repos
            .into_iter()
            .filter_map(|repo| validate_repo(&repo).ok())
            .collect()
    }
}

fn validate_repo(discovered: &DiscoveredRepo) -> Result<RepoInfo> {
    let repo = Repository::open(&discovered.path)
        .with_context(|| format!("opening repo at {}", discovered.path.display()))?;

    let head_oid = repo
        .head()
        .ok()
        .and_then(|h| h.target().map(|o| o.to_string()));

    let remote_url = repo
        .find_remote("origin")
        .ok()
        .and_then(|r| r.url().map(|u| u.to_string()));

    let default_branch = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(|s| s.to_string()));

    let name = discovered
        .path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| discovered.path.display().to_string());

    Ok(RepoInfo {
        path: discovered.path.clone(),
        name,
        remote_url,
        head_oid,
        default_branch,
        commit_count: 0,
        last_commit_at: None,
        authors: Vec::new(),
    })
}

pub fn scan_repositories(config: &Config) -> Result<Vec<RepoInfo>> {
    let discovered = discover_repos(config)?;
    Ok(validate_repos(discovered, config.parallel))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    fn init_repo(path: &Path) {
        let repo = Repository::init(path).unwrap();
        let sig = git2::Signature::now("Test Author", "test@example.com").unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.write_tree().unwrap()
        };
        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            "initial commit",
            &repo.find_tree(tree_id).unwrap(),
            &[],
        )
        .unwrap();
    }

    #[test]
    fn discovers_nested_repos() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let repo_path = root.join("project-a");
        fs::create_dir_all(&repo_path).unwrap();
        init_repo(&repo_path);

        let nested = root.join("project-a").join("vendor").join("lib-b");
        fs::create_dir_all(&nested).unwrap();
        init_repo(&nested);

        let config = Config {
            scan_paths: vec![root.to_path_buf()],
            exclude: vec!["target".into()],
            ..Config::default()
        };

        let discovered = discover_repos(&config).unwrap();
        assert_eq!(discovered.len(), 2);
    }

    #[test]
    fn excludes_directories() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let excluded = root.join("node_modules").join("pkg");
        fs::create_dir_all(&excluded).unwrap();
        init_repo(&excluded);

        let config = Config {
            scan_paths: vec![root.to_path_buf()],
            ..Config::default()
        };

        let discovered = discover_repos(&config).unwrap();
        assert!(discovered.is_empty());
    }
}

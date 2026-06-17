use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub struct AuthorKey {
    pub name: String,
    pub email: String,
}

impl AuthorKey {
    pub fn display_name(&self) -> &str {
        if self.name.is_empty() {
            &self.email
        } else {
            &self.name
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitRecord {
    pub repo_path: PathBuf,
    pub repo_name: String,
    pub oid: String,
    pub parents: Vec<String>,
    pub author: AuthorKey,
    pub timestamp: DateTime<Local>,
    pub message: String,
    pub message_length: usize,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInfo {
    pub path: PathBuf,
    pub name: String,
    pub remote_url: Option<String>,
    pub head_oid: Option<String>,
    pub default_branch: Option<String>,
    pub commit_count: usize,
    pub last_commit_at: Option<DateTime<Local>>,
    pub authors: Vec<AuthorKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub repos: Vec<RepoInfo>,
    pub commits: Vec<CommitRecord>,
    pub scan_paths: Vec<PathBuf>,
    pub scanned_at: DateTime<Local>,
}

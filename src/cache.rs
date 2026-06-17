use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedRepo {
    pub path: PathBuf,
    pub remote_url: Option<String>,
    pub head_oid: Option<String>,
    pub last_scan: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStore {
    pub repos: HashMap<String, CachedRepo>,
}

impl CacheStore {
    pub fn cache_file() -> Option<PathBuf> {
        Config::cache_dir().map(|d| d.join("cache.json"))
    }

    pub fn load() -> Result<Self> {
        let Some(path) = Self::cache_file() else {
            return Ok(Self::default());
        };
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents =
            fs::read_to_string(&path).with_context(|| format!("reading cache at {}", path.display()))?;
        let store: CacheStore =
            serde_json::from_str(&contents).with_context(|| format!("parsing cache at {}", path.display()))?;
        Ok(store)
    }

    pub fn save(&self) -> Result<()> {
        let Some(path) = Self::cache_file() else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(&path, contents).with_context(|| format!("writing cache at {}", path.display()))?;
        Ok(())
    }

    pub fn get(&self, path: &Path) -> Option<&CachedRepo> {
        self.repos.get(&path_key(path))
    }

    pub fn is_fresh(&self, path: &Path, ttl_hours: u64, head_oid: Option<&str>) -> bool {
        let Some(entry) = self.get(path) else {
            return false;
        };
        let age = Utc::now().signed_duration_since(entry.last_scan);
        if age.num_hours() >= ttl_hours as i64 {
            return false;
        }
        match (head_oid, entry.head_oid.as_deref()) {
            (Some(current), Some(cached)) => current == cached,
            _ => false,
        }
    }

    pub fn upsert(&mut self, path: PathBuf, remote_url: Option<String>, head_oid: Option<String>) {
        let key = path_key(&path);
        self.repos.insert(
            key,
            CachedRepo {
                path,
                remote_url,
                head_oid,
                last_scan: Utc::now(),
            },
        );
    }
}

fn path_key(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_when_head_matches_and_within_ttl() {
        let mut store = CacheStore::default();
        store.upsert(
            PathBuf::from("/tmp/repo"),
            None,
            Some("abc123".into()),
        );
        assert!(store.is_fresh(Path::new("/tmp/repo"), 24, Some("abc123")));
        assert!(!store.is_fresh(Path::new("/tmp/repo"), 24, Some("different")));
    }
}

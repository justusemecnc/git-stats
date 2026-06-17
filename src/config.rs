use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub scan_paths: Vec<PathBuf>,
    pub exclude: Vec<String>,
    pub max_depth: Option<usize>,
    pub parallel: bool,
    pub cache: CacheConfig,
    pub display: DisplayConfig,
    pub max_commits_per_repo: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub ttl_hours: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub theme: String,
    pub refresh_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scan_paths: vec![std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))],
            exclude: vec![
                "node_modules".into(),
                "target".into(),
                ".cargo".into(),
                ".git".into(),
            ],
            max_depth: None,
            parallel: true,
            cache: CacheConfig {
                enabled: true,
                ttl_hours: 24,
            },
            display: DisplayConfig {
                theme: "dark".into(),
                refresh_secs: 30,
            },
            max_commits_per_repo: None,
        }
    }
}

impl Config {
    pub fn config_dir() -> Option<PathBuf> {
        ProjectDirs::from("", "", "git-stats").map(|d| d.config_dir().to_path_buf())
    }

    pub fn cache_dir() -> Option<PathBuf> {
        ProjectDirs::from("", "", "git-stats").map(|d| d.cache_dir().to_path_buf())
    }

    pub fn config_path() -> Option<PathBuf> {
        Self::config_dir().map(|d| d.join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let mut config = Self::default();
        if let Some(path) = Self::config_path() {
            if path.exists() {
                let contents = fs::read_to_string(&path)
                    .with_context(|| format!("reading config at {}", path.display()))?;
                let file_config: Config = toml::from_str(&contents)
                    .with_context(|| format!("parsing config at {}", path.display()))?;
                config.merge(file_config);
            }
        }
        Ok(config)
    }

    fn merge(&mut self, other: Config) {
        if !other.scan_paths.is_empty() {
            self.scan_paths = other.scan_paths;
        }
        if !other.exclude.is_empty() {
            self.exclude = other.exclude;
        }
        if other.max_depth.is_some() {
            self.max_depth = other.max_depth;
        }
        self.parallel = other.parallel;
        self.cache = other.cache;
        self.display = other.display;
        if other.max_commits_per_repo.is_some() {
            self.max_commits_per_repo = other.max_commits_per_repo;
        }
    }

    pub fn with_scan_paths(mut self, paths: Vec<PathBuf>) -> Self {
        if !paths.is_empty() {
            self.scan_paths = paths;
        }
        self
    }

    pub fn with_no_cache(mut self) -> Self {
        self.cache.enabled = false;
        self
    }

    pub fn is_excluded(&self, name: &str) -> bool {
        self.exclude.iter().any(|pattern| {
            if pattern.contains('*') {
                glob_match(pattern, name)
            } else {
                name == pattern
            }
        })
    }
}

fn glob_match(pattern: &str, name: &str) -> bool {
    if let Some(stripped) = pattern.strip_prefix('*') {
        name.ends_with(stripped)
    } else if let Some(stripped) = pattern.strip_suffix('*') {
        name.starts_with(stripped)
    } else {
        name == pattern
    }
}

pub fn ensure_config_dirs() -> Result<()> {
    if let Some(dir) = Config::config_dir() {
        fs::create_dir_all(&dir).with_context(|| format!("creating config dir {}", dir.display()))?;
    }
    if let Some(dir) = Config::cache_dir() {
        fs::create_dir_all(&dir).with_context(|| format!("creating cache dir {}", dir.display()))?;
    }
    Ok(())
}

pub fn example_config_path() -> Option<PathBuf> {
    Config::config_path()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exclude_exact_match() {
        let config = Config::default();
        assert!(config.is_excluded("node_modules"));
        assert!(!config.is_excluded("my-node_modules"));
    }

    #[test]
    fn exclude_glob_match() {
        let mut config = Config::default();
        config.exclude.push("*.tmp".into());
        assert!(config.is_excluded("foo.tmp"));
        assert!(!config.is_excluded("foo.txt"));
    }
}

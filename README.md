# git-stats

Terminal Git activity dashboard that scans, analyzes, and visualizes repository activity across a projects folder.

## Features

- Recursive repository discovery with exclude patterns and parallel validation
- Commit extraction via **git2** (no shelling out to `git`)
- Author, temporal, velocity, and repository health metrics
- Interactive TUI with dashboard, authors, repos, timeline, and achievements views
- Output modes: TUI (default), summary, JSON, Markdown
- Contribution heatmap, watch mode, and achievement system
- Disk cache with HEAD OID fast path

## Installation

```bash
cargo install --path .
```

Requires Rust 1.70+. Uses vendored libgit2 via the `git2` crate.

## Usage

```bash
# Interactive dashboard (default)
git-stats

# One-line summary
git-stats summary

# JSON export
git-stats export --json

# Markdown report
git-stats export --markdown -o report.md

# Scan a specific directory
git-stats --path ~/projects summary

# Disable cache
git-stats --no-cache summary
```

## Configuration

Config file: `~/.config/git-stats/config.toml` (Windows: `%APPDATA%\git-stats\config.toml`)

```toml
scan_paths = ["D:/projects", "C:/dev"]
exclude = ["node_modules", "target", ".cargo", ".git"]
max_depth = 8
parallel = true
max_commits_per_repo = 5000

[cache]
enabled = true
ttl_hours = 24

[display]
theme = "dark"   # or "light"
refresh_secs = 30
```

## TUI Keybindings

| Key | Action |
|-----|--------|
| Tab / Shift+Tab | Switch views |
| ↑ / ↓ | Navigate lists |
| Enter | Drill into author/repo detail |
| Esc | Close detail panel |
| r | Refresh scan |
| w | Toggle watch mode |
| s | Sort authors by name (Authors view) |
| q | Quit |

## Achievements

| Badge | Criteria |
|-------|----------|
| Night Owl | 50%+ commits between midnight and 5 AM |
| Early Bird | 50%+ commits before 8 AM |
| Marathon | Commits on 30+ distinct days |
| Lone Wolf | Sole contributor in at least one repo |
| Commitizen | Average commit message length > 50 chars |

## Development

```bash
cargo test
cargo bench
cargo run -- summary --path .
```

## Stretch goals

- GitHub/GitLab remote comparison via API
- WebAssembly dashboard
- Slack weekly stats bot

## Author

justusemecnc

## License

MIT

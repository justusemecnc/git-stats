# git-stats

Terminal Git activity dashboard. Scan a projects folder, analyze commit history, and explore stats in a TUI or export them from the shell.

## What it does

- Discovers Git repos recursively with exclude patterns and parallel validation
- Extracts commit metadata via **git2** (no shelling out to `git`)
- Author metrics, temporal patterns, velocity, and repository health
- Interactive TUI — dashboard, authors, repos, timeline, and achievements
- One-line summary, JSON export, or Markdown report from the CLI
- Contribution heatmap, watch mode, and achievement system
- Disk cache with HEAD OID fast path

## Requirements

- Rust 1.70+
- Cargo

Uses vendored libgit2 via the `git2` crate.

## Build

```powershell
cargo build --release
./target/release/git-stats.exe summary --path D:/Portfolio
```

```bash
cargo build --release
./target/release/git-stats summary --path ~/projects
```

Install globally:

```bash
cargo install --path .
```

## Try it

Scan a single repo or an entire projects folder:

```powershell
cargo run -- summary --path D:/Portfolio/portfolio-website
cargo run -- export --json --path D:/Portfolio > stats.json
cargo run -- --path D:/Portfolio
```

```bash
cargo run -- summary --path ~/projects
cargo run -- export --markdown -o report.md --path ~/projects
cargo run -- --path ~/projects
```

## Docs

### CLI

| Command | Description |
|---------|-------------|
| `git-stats` | Interactive TUI (default) |
| `git-stats summary` | One-line overview |
| `git-stats export --json` | JSON to stdout |
| `git-stats export --markdown -o report.md` | Markdown report |
| `git-stats --path DIR` | Override scan path |
| `git-stats --no-cache` | Skip disk cache |

### TUI keybindings

| Key | Action |
|-----|--------|
| Tab / Shift+Tab | Switch views |
| ↑ / ↓ | Navigate lists |
| Enter | Author/repo detail |
| Esc | Close detail panel |
| r | Refresh scan |
| w | Toggle watch mode |
| s | Sort authors by name |
| q | Quit |

### Configuration

Config file: `~/.config/git-stats/config.toml` (Windows: `%APPDATA%\git-stats\config.toml`)

```toml
scan_paths = ["D:/projects", "C:/dev"]
exclude = ["node_modules", "target", ".cargo"]
max_depth = 8
parallel = true
max_commits_per_repo = 5000

[cache]
enabled = true
ttl_hours = 24

[display]
theme = "dark"
refresh_secs = 30
```

### Achievements

| Badge | Criteria |
|-------|----------|
| Night Owl | 50%+ commits between midnight and 5 AM |
| Early Bird | 50%+ commits before 8 AM |
| Marathon | Commits on 30+ distinct days |
| Lone Wolf | Sole contributor in at least one repo |
| Commitizen | Average commit message length > 50 chars |

## Source

Modules in `src/` — `scanner`, `git/extract`, `stats`, `ui`, `output`, `achievements`. Tests in `tests/`, benchmarks in `benches/`.

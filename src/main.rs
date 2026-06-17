use anyhow::Result;
use git_stats::cli::{Cli, Commands};
use git_stats::config::{ensure_config_dirs, Config};
use git_stats::output;
use git_stats::pipeline::run_scan;
use git_stats::ui;

use clap::Parser;

fn main() -> Result<()> {
    let cli = Cli::parse();
    ensure_config_dirs()?;

    let mut config = Config::load()?;
    if cli.no_cache {
        config = config.with_no_cache();
    }
    if let Some(path) = cli.path.clone() {
        config = config.with_scan_paths(vec![path]);
    }

    match cli.command {
        None | Some(Commands::Tui) => ui::run(config),
        Some(Commands::Summary) => {
            let output = run_scan(&config)?;
            output::summary::print_summary(&output.stats);
            Ok(())
        }
        Some(Commands::Export { json: _, markdown, output }) => {
            let data = run_scan(&config)?;
            if markdown {
                let path = output.unwrap_or_else(|| "git-stats-report.md".into());
                output::markdown::write_markdown(&path, &data.scan, &data.stats, &data.achievements)?;
            } else {
                output::json::print_json(&data.scan, &data.stats, &data.achievements)?;
            }
            Ok(())
        }
        Some(Commands::Scan) => {
            let data = run_scan(&config)?;
            println!("Found {} repositories", data.stats.total_repos);
            Ok(())
        }
    }
}

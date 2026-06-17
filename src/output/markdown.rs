use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::achievements::Achievement;
use crate::models::ScanResult;
use crate::stats::GlobalStats;

pub fn write_markdown(
    path: &Path,
    scan: &ScanResult,
    stats: &GlobalStats,
    achievements: &[Achievement],
) -> Result<()> {
    let mut out = String::new();
    out.push_str("# Git-Stats Report\n\n");
    out.push_str(&format!("Generated: {}\n\n", scan.scanned_at.format("%Y-%m-%d %H:%M")));

    out.push_str("## Overview\n\n");
    out.push_str(&format!("- Repositories: {}\n", stats.total_repos));
    out.push_str(&format!("- Commits: {}\n", stats.total_commits));
    out.push_str(&format!("- Authors: {}\n", stats.total_authors));
    out.push_str(&format!("- Stale repos (>30d): {}\n", stats.stale_repos));
    out.push_str(&format!("- Avg message length: {:.1}\n", stats.avg_message_length));
    out.push_str(&format!(
        "- Velocity: 7d={} 14d={} 30d={}\n\n",
        stats.velocity_7d, stats.velocity_14d, stats.velocity_30d
    ));

    out.push_str("## Top Contributors\n\n");
    out.push_str("| Author | Commits | Avg Msg Len |\n");
    out.push_str("|--------|---------|-------------|\n");
    for author in stats.authors.iter().take(10) {
        out.push_str(&format!(
            "| {} | {} | {:.1} |\n",
            author.author.display_name(),
            author.commit_count,
            author.avg_message_length()
        ));
    }
    out.push('\n');

    out.push_str("## Repositories\n\n");
    out.push_str("| Repo | Commits | Authors | Stale |\n");
    out.push_str("|------|---------|---------|-------|\n");
    for repo in stats.repos.iter().take(20) {
        out.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            repo.repo_name,
            repo.commit_count,
            repo.author_count,
            if repo.is_stale { "yes" } else { "no" }
        ));
    }
    out.push('\n');

    out.push_str("## Activity Heatmap\n\n");
    out.push_str("```\n");
    out.push_str(&render_ascii_heatmap(stats));
    out.push_str("```\n\n");

    out.push_str("## Achievements\n\n");
    for ach in achievements {
        let mark = if ach.unlocked { "[x]" } else { "[ ]" };
        out.push_str(&format!("- {mark} **{}**: {}\n", ach.title, ach.description));
    }

    fs::write(path, out).with_context(|| format!("writing markdown to {}", path.display()))?;
    Ok(())
}

fn render_ascii_heatmap(stats: &GlobalStats) -> String {
    let mut lines = Vec::new();
    let mut week = String::new();
    for (i, (date, level)) in stats.heatmap.cells.iter().enumerate() {
        let ch = match level {
            0 => '.',
            1 => '+',
            2 => '*',
            _ => '#',
        };
        week.push(ch);
        if date.weekday().num_days_from_monday() == 6 || i == stats.heatmap.cells.len() - 1 {
            lines.push(week.clone());
            week.clear();
        }
    }
    lines.join("\n")
}

use chrono::Datelike;

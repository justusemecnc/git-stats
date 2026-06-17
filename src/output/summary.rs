use crate::stats::GlobalStats;

pub fn print_summary(stats: &GlobalStats) {
    println!(
        "{} repos | {} commits | {} authors | {} stale",
        stats.total_repos, stats.total_commits, stats.total_authors, stats.stale_repos
    );
}

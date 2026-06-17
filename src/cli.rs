use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "git-stats", about = "Terminal Git activity dashboard", author = "justusemecnc")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(long, global = true)]
    pub path: Option<PathBuf>,

    #[arg(long, global = true)]
    pub no_cache: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Interactive TUI dashboard
    Tui,
    /// One-line overview
    Summary,
    /// Export data
    Export {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        markdown: bool,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Scan and print repo count
    Scan,
}

impl Cli {
    pub fn is_tui(&self) -> bool {
        matches!(self.command, None | Some(Commands::Tui))
    }
}

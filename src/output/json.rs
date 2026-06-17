use anyhow::Result;
use serde::Serialize;

use crate::achievements::Achievement;
use crate::models::ScanResult;
use crate::stats::GlobalStats;

#[derive(Serialize)]
pub struct ExportData<'a> {
    pub scan: &'a ScanResult,
    pub stats: &'a GlobalStats,
    pub achievements: &'a [Achievement],
}

pub fn print_json(scan: &ScanResult, stats: &GlobalStats, achievements: &[Achievement]) -> Result<()> {
    let data = ExportData {
        scan,
        stats,
        achievements,
    };
    println!("{}", serde_json::to_string_pretty(&data)?);
    Ok(())
}

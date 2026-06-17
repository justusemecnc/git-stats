use chrono::Datelike;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::palette::Palette;
use crate::stats::HeatmapData;

const WEEKDAY_LABELS: [&str; 7] = ["M", "T", "W", "T", "F", "S", "S"];

pub fn render(f: &mut Frame, area: ratatui::layout::Rect, palette: Palette, heatmap: &HeatmapData) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Contribution heatmap (365d) ")
        .border_style(Style::default().fg(palette.secondary));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if heatmap.cells.is_empty() {
        let msg = Paragraph::new("No activity data").style(Style::default().fg(palette.muted));
        f.render_widget(msg, inner);
        return;
    }

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(inner);

    let day_labels: Vec<Line> = WEEKDAY_LABELS
        .iter()
        .map(|l| Line::from(Span::styled(*l, Style::default().fg(palette.muted))))
        .collect();
    f.render_widget(Paragraph::new(day_labels), cols[0]);

    let mut weeks: Vec<Vec<(char, u8)>> = Vec::new();
    let mut current_week: Vec<(char, u8)> = Vec::new();

    let first = heatmap.cells.first().map(|(d, _)| *d);
    if let Some(start) = first {
        let pad = start.weekday().num_days_from_monday() as usize;
        for _ in 0..pad {
            current_week.push(('.', 0));
        }
    }

    for (date, level) in &heatmap.cells {
        let ch = level_char(*level);
        current_week.push((ch, *level));
        if date.weekday().num_days_from_sunday() == 0 {
            weeks.push(current_week.clone());
            current_week.clear();
        }
    }
    if !current_week.is_empty() {
        weeks.push(current_week);
    }

    let mut lines = Vec::new();
    for row in 0..7 {
        let mut spans = Vec::new();
        for week in &weeks {
            if let Some((ch, level)) = week.get(row) {
                spans.push(Span::styled(
                    format!("{ch} "),
                    Style::default().fg(palette.heatmap_color(*level)),
                ));
            } else {
                spans.push(Span::raw("  "));
            }
        }
        lines.push(Line::from(spans));
    }

    f.render_widget(Paragraph::new(lines), cols[1]);
}

fn level_char(level: u8) -> char {
    match level {
        0 => '·',
        1 => '▁',
        2 => '▃',
        _ => '█',
    }
}

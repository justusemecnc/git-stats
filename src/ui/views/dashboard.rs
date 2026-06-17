use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Sparkline},
    Frame,
};

use crate::palette::Palette;
use crate::pipeline::PipelineOutput;

use super::super::app::App;
use super::super::widgets::heatmap;

pub fn render(f: &mut Frame, area: ratatui::layout::Rect, app: &App, data: &PipelineOutput) {
    let stats = &data.stats;
    let palette = app.palette;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Min(6),
            Constraint::Length(3),
            Constraint::Length(5),
        ])
        .split(area);

    let cards = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(chunks[0]);

    render_card(f, cards[0], palette, "Repos", &stats.total_repos.to_string());
    render_card(f, cards[1], palette, "Commits", &stats.total_commits.to_string());
    render_card(f, cards[2], palette, "Authors", &stats.total_authors.to_string());
    render_card(f, cards[3], palette, "Stale", &stats.stale_repos.to_string());

    heatmap::render(f, chunks[1], palette, &stats.heatmap);

    let spark_data: Vec<u64> = stats
        .daily_commits
        .iter()
        .rev()
        .take(60)
        .map(|(_, c)| *c as u64)
        .collect();

    let spark = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Recent activity (60d) ")
                .border_style(Style::default().fg(palette.secondary)),
        )
        .data(&spark_data)
        .style(Style::default().fg(palette.primary));

    f.render_widget(spark, chunks[2]);

    let info = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Peak hour: ", Style::default().fg(palette.muted)),
            Span::styled(format!("{}:00", stats.peak_hour), Style::default().fg(palette.text)),
        ]),
        Line::from(vec![
            Span::styled("Peak day: ", Style::default().fg(palette.muted)),
            Span::styled(&stats.peak_weekday, Style::default().fg(palette.text)),
        ]),
        Line::from(vec![
            Span::styled("Velocity 7/14/30d: ", Style::default().fg(palette.muted)),
            Span::styled(
                format!(
                    "{} / {} / {}",
                    stats.velocity_7d, stats.velocity_14d, stats.velocity_30d
                ),
                Style::default().fg(palette.text),
            ),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Insights ")
            .border_style(Style::default().fg(palette.secondary)),
    );
    f.render_widget(info, chunks[3]);
}

fn render_card(f: &mut Frame, area: ratatui::layout::Rect, palette: Palette, label: &str, value: &str) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {label} "))
        .border_style(Style::default().fg(palette.secondary));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines = vec![
        Line::from(Span::styled(
            value,
            Style::default()
                .fg(palette.primary)
                .add_modifier(Modifier::BOLD),
        )),
    ];
    f.render_widget(Paragraph::new(lines), inner);
}

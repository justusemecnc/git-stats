use ratatui::{
    style::Style,
    text::Line,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame,
};

use crate::pipeline::PipelineOutput;

use super::super::app::App;

pub fn render(f: &mut Frame, area: ratatui::layout::Rect, app: &App, data: &PipelineOutput) {
    let palette = app.palette;
    let daily = &data.stats.daily_commits;

    let values: Vec<(f64, f64)> = daily
        .iter()
        .enumerate()
        .map(|(i, (_, count))| (i as f64, *count as f64))
        .collect();

    let max_y = daily.iter().map(|(_, c)| *c).max().unwrap_or(1).max(1) as f64;

    let datasets = vec![Dataset::default()
        .name("commits")
        .marker(ratatui::symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(palette.primary))
        .data(&values)];

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Commit frequency (daily) ")
                .border_style(Style::default().fg(palette.secondary)),
        )
        .x_axis(
            Axis::default()
                .bounds([0.0, values.len().max(1) as f64])
                .labels(vec![
                    Line::from("start"),
                    Line::from("now"),
                ]),
        )
        .y_axis(
            Axis::default()
                .bounds([0.0, max_y])
                .labels(vec![
                    Line::from("0"),
                    Line::from(max_y.to_string()),
                ]),
        );

    f.render_widget(chart, area);
}

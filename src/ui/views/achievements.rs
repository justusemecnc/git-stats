use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::pipeline::PipelineOutput;

use super::super::app::App;

pub fn render(f: &mut Frame, area: ratatui::layout::Rect, app: &App, data: &PipelineOutput) {
    let palette = app.palette;

    let items: Vec<ListItem> = data
        .achievements
        .iter()
        .enumerate()
        .map(|(i, ach)| {
            let icon = if ach.unlocked { "✓" } else { "○" };
            let style = if ach.unlocked {
                Style::default().fg(palette.success)
            } else if i == app.selected {
                Style::default().fg(palette.highlight)
            } else {
                Style::default().fg(palette.muted)
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{icon} "), style.add_modifier(Modifier::BOLD)),
                Span::styled(&ach.title, Style::default().fg(palette.text)),
                Span::styled(format!(" — {}", ach.description), Style::default().fg(palette.muted)),
            ]))
        })
        .collect();

    let unlocked = data.achievements.iter().filter(|a| a.unlocked).count();
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Achievements ({unlocked}/{}) ", data.achievements.len()))
            .border_style(Style::default().fg(palette.secondary)),
    );

    f.render_widget(list, area);
}

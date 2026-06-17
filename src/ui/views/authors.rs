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
    let mut authors = data.stats.authors.clone();
    if app.author_sort_by_name {
        authors.sort_by(|a, b| a.author.display_name().cmp(b.author.display_name()));
    }

    if app.selected >= authors.len() && !authors.is_empty() {
        // clamp handled in draw cycle via selection
    }

    let max_commits = authors.first().map(|a| a.commit_count).unwrap_or(1).max(1);

    let items: Vec<ListItem> = authors
        .iter()
        .enumerate()
        .map(|(i, author)| {
            let bar_len = (author.commit_count * 20 / max_commits).max(1);
            let bar: String = std::iter::repeat('█').take(bar_len).collect();
            let line = format!(
                "{:<20} {:>5}  {} {:.1}",
                truncate(author.author.display_name(), 20),
                author.commit_count,
                bar,
                author.avg_message_length()
            );
            let style = if i == app.selected {
                Style::default()
                    .fg(palette.highlight)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(palette.text)
            };
            ListItem::new(Line::from(Span::styled(line, style)))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Authors (s=sort, Enter=detail) ")
            .border_style(Style::default().fg(palette.secondary)),
    );

    f.render_widget(list, area);
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}

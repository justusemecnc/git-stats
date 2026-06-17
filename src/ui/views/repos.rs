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
        .stats
        .repos
        .iter()
        .enumerate()
        .map(|(i, repo)| {
            let stale = if repo.is_stale { " [stale]" } else { "" };
            let highlight = app.highlighted_repos.contains_key(&repo.repo_name);
            let new_tag = if highlight { " [NEW]" } else { "" };
            let last = repo
                .last_commit_at
                .map(|t| t.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "-".into());
            let line = format!(
                "{:<24} commits={:<5} authors={:<3} last={}{}",
                truncate(&repo.repo_name, 24),
                repo.commit_count,
                repo.author_count,
                last,
                stale
            );
            let mut style = if i == app.selected {
                Style::default()
                    .fg(palette.highlight)
                    .add_modifier(Modifier::BOLD)
            } else if repo.is_stale {
                Style::default().fg(palette.warning)
            } else {
                Style::default().fg(palette.text)
            };
            if highlight {
                style = style.fg(palette.accent).add_modifier(Modifier::BOLD);
            }
            ListItem::new(Line::from(vec![
                Span::styled(line, style),
                Span::styled(new_tag, Style::default().fg(palette.accent)),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Repositories (Enter=detail) ")
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

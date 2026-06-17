use std::collections::HashMap;
use std::time::Instant;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::config::Config;
use crate::palette::Palette;
use crate::pipeline::{run_scan, PipelineOutput};

use super::views;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Dashboard,
    Authors,
    Repos,
    Timeline,
    Achievements,
}

impl View {
    pub fn all() -> &'static [View] {
        &[
            View::Dashboard,
            View::Authors,
            View::Repos,
            View::Timeline,
            View::Achievements,
        ]
    }

    fn title(self) -> &'static str {
        match self {
            View::Dashboard => "Dashboard",
            View::Authors => "Authors",
            View::Repos => "Repos",
            View::Timeline => "Timeline",
            View::Achievements => "Achievements",
        }
    }
}

pub struct App {
    pub config: Config,
    pub palette: Palette,
    pub view: View,
    pub data: Option<PipelineOutput>,
    pub selected: usize,
    pub author_sort_by_name: bool,
    pub watch_mode: bool,
    pub should_quit: bool,
    pub loading: bool,
    pub status: String,
    pub last_refresh: Instant,
    pub prev_commit_counts: HashMap<String, usize>,
    pub highlighted_repos: HashMap<String, Instant>,
    pub detail: Option<String>,
}

impl App {
    pub fn new(config: Config, palette: Palette) -> Self {
        Self {
            config,
            palette,
            view: View::Dashboard,
            data: None,
            selected: 0,
            author_sort_by_name: false,
            watch_mode: false,
            should_quit: false,
            loading: false,
            status: "Press r to scan, q to quit".into(),
            last_refresh: Instant::now(),
            prev_commit_counts: HashMap::new(),
            highlighted_repos: HashMap::new(),
            detail: None,
        }
    }

    pub fn refresh(&mut self) -> anyhow::Result<()> {
        self.loading = true;
        self.status = "Scanning repositories...".into();
        match run_scan(&self.config) {
            Ok(output) => {
                self.prev_commit_counts = output
                    .stats
                    .repos
                    .iter()
                    .map(|r| (r.repo_name.clone(), r.commit_count))
                    .collect();
                self.data = Some(output);
                self.status = format!(
                    "Scanned {} repos | w=watch {} | r=refresh | Tab=views",
                    self.data.as_ref().map(|d| d.stats.total_repos).unwrap_or(0),
                    if self.watch_mode { "ON" } else { "OFF" }
                );
                self.last_refresh = Instant::now();
            }
            Err(e) => {
                self.status = format!("Error: {e:#}");
            }
        }
        self.loading = false;
        Ok(())
    }

    pub fn refresh_with_highlight(&mut self) -> anyhow::Result<()> {
        let prev = self.prev_commit_counts.clone();
        self.refresh()?;
        if let Some(data) = &self.data {
            for repo in &data.stats.repos {
                let old = prev.get(&repo.repo_name).copied().unwrap_or(0);
                if repo.commit_count > old {
                    self.highlighted_repos
                        .insert(repo.repo_name.clone(), Instant::now());
                }
            }
        }
        self.highlighted_repos
            .retain(|_, t| t.elapsed().as_secs() < 5);
        Ok(())
    }

    pub fn next_view(&mut self) {
        let views = View::all();
        let idx = views.iter().position(|v| *v == self.view).unwrap_or(0);
        self.view = views[(idx + 1) % views.len()];
        self.selected = 0;
        self.detail = None;
    }

    pub fn prev_view(&mut self) {
        let views = View::all();
        let idx = views.iter().position(|v| *v == self.view).unwrap_or(0);
        let new_idx = if idx == 0 { views.len() - 1 } else { idx - 1 };
        self.view = views[new_idx];
        self.selected = 0;
        self.detail = None;
    }

    pub fn draw(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(2),
            ])
            .split(f.area());

        let titles: Vec<Line> = View::all()
            .iter()
            .map(|v| {
                let style = if *v == self.view {
                    Style::default()
                        .fg(self.palette.primary)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.palette.muted)
                };
                Line::from(Span::styled(format!(" {} ", v.title()), style))
            })
            .collect();

        let tabs = Tabs::new(titles).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" git-stats ")
                .border_style(Style::default().fg(self.palette.secondary)),
        );
        f.render_widget(tabs, chunks[0]);

        if let Some(data) = &self.data {
            match self.view {
                View::Dashboard => views::dashboard::render(f, chunks[1], self, data),
                View::Authors => views::authors::render(f, chunks[1], self, data),
                View::Repos => views::repos::render(f, chunks[1], self, data),
                View::Timeline => views::timeline::render(f, chunks[1], self, data),
                View::Achievements => views::achievements::render(f, chunks[1], self, data),
            }
        } else {
            let msg = Paragraph::new("No data loaded. Press r to scan.")
                .style(Style::default().fg(self.palette.text));
            f.render_widget(msg, chunks[1]);
        }

        let status = if self.loading {
            "Loading..."
        } else {
            &self.status
        };
        let footer = Paragraph::new(status).style(Style::default().fg(self.palette.muted));
        f.render_widget(footer, chunks[2]);

        if let Some(detail) = &self.detail {
            self.draw_detail(f, detail);
        }
    }

    fn draw_detail(&self, f: &mut Frame, detail: &str) {
        let area = centered_rect(60, 40, f.area());
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Detail (Esc to close) ")
            .border_style(Style::default().fg(self.palette.accent));
        let p = Paragraph::new(detail).block(block).style(Style::default().fg(self.palette.text));
        f.render_widget(p, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

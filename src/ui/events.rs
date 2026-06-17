use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

use super::app::App;

pub fn handle_events(app: &mut App, timeout: Duration) -> anyhow::Result<bool> {
    if event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                return Ok(false);
            }
            match key.code {
                KeyCode::Char('q') => app.should_quit = true,
                KeyCode::Char('r') => {
                    app.refresh()?;
                }
                KeyCode::Char('w') => {
                    app.watch_mode = !app.watch_mode;
                    app.status = format!(
                        "Watch mode {}",
                        if app.watch_mode { "enabled" } else { "disabled" }
                    );
                }
                KeyCode::Tab => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        app.prev_view();
                    } else {
                        app.next_view();
                    }
                }
                KeyCode::Char('s') if matches!(app.view, super::app::View::Authors) => {
                    app.author_sort_by_name = !app.author_sort_by_name;
                }
                KeyCode::Up => {
                    if app.selected > 0 {
                        app.selected -= 1;
                    }
                }
                KeyCode::Down => {
                    let max = app.list_len();
                    if max > 0 && app.selected + 1 < max {
                        app.selected += 1;
                    }
                }
                KeyCode::Enter => app.open_detail(),
                KeyCode::Esc => app.detail = None,
                _ => {}
            }
        }
    }
    Ok(false)
}

impl App {
    pub fn open_detail(&mut self) {
        let Some(data) = &self.data else {
            return;
        };
        match self.view {
            super::app::View::Authors => {
                if let Some(author) = data.stats.authors.get(self.selected) {
                    self.detail = Some(format!(
                        "Author: {}\nEmail: {}\nCommits: {}\nAvg message: {:.1}\nRepos: {}",
                        author.author.display_name(),
                        author.author.email,
                        author.commit_count,
                        author.avg_message_length(),
                        author.repos.len()
                    ));
                }
            }
            super::app::View::Repos => {
                if let Some(repo) = data.stats.repos.get(self.selected) {
                    self.detail = Some(format!(
                        "Repo: {}\nPath: {}\nCommits: {}\nAuthors: {}\nStale: {}\nRemote: {}",
                        repo.repo_name,
                        repo.repo_path,
                        repo.commit_count,
                        repo.author_count,
                        repo.is_stale,
                        repo.remote_url.as_deref().unwrap_or("-")
                    ));
                }
            }
            _ => {}
        }
    }

    pub fn list_len(&self) -> usize {
        let Some(data) = &self.data else {
            return 0;
        };
        match self.view {
            super::app::View::Authors => data.stats.authors.len(),
            super::app::View::Repos => data.stats.repos.len(),
            super::app::View::Achievements => data.achievements.len(),
            _ => 0,
        }
    }
}

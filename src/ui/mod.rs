mod app;
mod events;
mod views;
mod widgets;

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::{Duration, Instant};

use crate::config::Config;
use crate::palette::Palette;

use app::App;

pub fn run(config: Config) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let palette = Palette::from_theme(&config.display.theme);
    let refresh_secs = config.display.refresh_secs;

    let mut app = App::new(config.clone(), palette);
    app.refresh()?;

    let tick = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| app.draw(f))?;

        let timeout = tick.saturating_sub(last_tick.elapsed());
        if events::handle_events(&mut app, timeout)? {
            break;
        }

        if last_tick.elapsed() >= tick {
            if app.watch_mode {
                let interval = Duration::from_secs(refresh_secs);
                if app.last_refresh.elapsed() >= interval {
                    app.refresh_with_highlight()?;
                }
            }
            last_tick = Instant::now();
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

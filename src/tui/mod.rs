mod action;
mod app;
mod input;
mod views;
mod widgets;

use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;

use app::App;
use input::map_key;

const TICK_RATE: Duration = Duration::from_millis(50); // ~20 FPS for animations

pub fn run() -> io::Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut app = App::new();
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|frame| app.render(frame))?;

        let timeout = TICK_RATE.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if let Some(action) = map_key(key) {
                        if !app.handle(action) {
                            break;
                        }
                    }
                }
            }
        }

        if last_tick.elapsed() >= TICK_RATE {
            app.tick();
            last_tick = Instant::now();
        }
    }

    io::stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

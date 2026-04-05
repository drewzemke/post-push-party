mod action;
mod app;
mod input;
mod views;
mod widgets;

use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal as RatatuiTerminal, prelude::*};

use app::App;
use input::map_key;

use crate::{state::State, storage::DbConnection};

pub type Terminal = RatatuiTerminal<CrosstermBackend<io::Stdout>>;

const TICK_RATE: Duration = Duration::from_millis(50); // ~20 FPS for animations

fn enter_tui() -> io::Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    Ok(())
}

fn leave_tui() -> io::Result<()> {
    io::stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

pub fn run(state: &mut State, conn: &DbConnection) -> anyhow::Result<()> {
    enter_tui()?;
    let mut terminal: Terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut app = App::new(state, conn);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|frame| app.render(frame))?;

        let timeout = TICK_RATE.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
            && let Some(action) = map_key(key)
            && !app.handle(action)
        {
            break;
        }

        // run a game if there is one to run

        if let Some(game) = app.take_pending_game() {
            leave_tui()?;
            game.run(&mut terminal)?;
            enter_tui()?;
            terminal.clear()?;
        }

        if last_tick.elapsed() >= TICK_RATE {
            app.tick();
            last_tick = Instant::now();
        }
    }

    app.save();
    leave_tui()?;
    Ok(())
}

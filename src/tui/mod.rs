use std::{
    io,
    time::{Duration, Instant},
};

use anyhow::Result;
use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use ratatui::{Terminal as RatatuiTerminal, prelude::*};

#[cfg(feature = "dev")]
use crossterm::style::SetBackgroundColor;

use crate::{
    game::wallet::UserWallet,
    state::State,
    storage::{DbConnection, game_state},
    tui::transition::Transition,
};

mod action;
mod app;
mod input;
mod transition;
mod views;
mod widgets;

use app::App;
use input::map_key;

pub type Terminal = RatatuiTerminal<CrosstermBackend<io::Stdout>>;

const TICK_RATE: Duration = Duration::from_millis(50); // ~20 FPS for animations

pub fn get_terminal() -> Result<Terminal> {
    Ok(Terminal::new(CrosstermBackend::new(io::stdout()))?)
}

#[cfg(feature = "dev")]
pub fn clear_bg_color(color: (u8, u8, u8)) -> Result<()> {
    Ok(execute!(
        io::stdout(),
        SetBackgroundColor(color.into()),
        Clear(ClearType::All)
    )?)
}

pub fn enter_tui() -> io::Result<()> {
    enable_raw_mode()?;
    execute!(
        io::stdout(),
        EnterAlternateScreen,
        Clear(ClearType::All),
        Hide
    )?;
    Ok(())
}

pub fn leave_tui() -> io::Result<()> {
    execute!(io::stdout(), LeaveAlternateScreen, Show)?;
    disable_raw_mode()?;
    Ok(())
}

pub fn run(state: &mut State, conn: &DbConnection) -> anyhow::Result<()> {
    enter_tui()?;
    let mut terminal = get_terminal()?;

    let mut app = App::new(state, conn, terminal.size()?);
    let mut last_tick = Instant::now();

    loop {
        let frame = terminal.draw(|frame| app.render(frame))?;

        let timeout = TICK_RATE.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            let event = event::read()?;

            if let Event::Key(key) = event
                && key.kind == KeyEventKind::Press
                && let Some(action) = map_key(key)
                && !app.handle(action)
            {
                break;
            } else if let Event::Resize(cols, rows) = event {
                let size = Size::new(cols, rows);
                app.update_size(size);
            }
        }

        // run a game if there is one to run
        if let Some(game) = app.take_pending_game() {
            let mut game_state = game_state::load(conn, game.id())?;
            let mut wallet = UserWallet::new(conn);

            let mut transition = Transition::new(frame.buffer.clone());

            transition.transition_to(game.clear_color(), &mut terminal)?;
            let game_result = game.run(&mut terminal, &mut wallet, &mut game_state);
            if let Err(err) = game_result {
                // FIXME: do we also refund the token?
                app.set_error(err.to_string());
            }
            terminal.clear()?;
            transition.transition_from(game.clear_color(), &mut terminal)?;

            if let Some(state) = game_state {
                game_state::save(conn, game.id(), &state)?;
            }

            // reload tui state in case points were updated
            app.reload_state()?;
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

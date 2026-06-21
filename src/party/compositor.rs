use std::io::{IsTerminal, Write as _};

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{BeginSynchronizedUpdate, EndSynchronizedUpdate},
};

use super::FullscreenPartyRenderer;
use crate::tui;

// NOTE: this roughly determines the framerate
const POLL_TIME: std::time::Duration = std::time::Duration::from_millis(10);

pub fn run(mut parties: Vec<Box<dyn FullscreenPartyRenderer>>) -> anyhow::Result<()> {
    // make sure if stdout is a TTY. if it isn't (eg. the output of party
    // is being piped into something), just silently return
    if !std::io::stdout().is_terminal() {
        return Ok(());
    }

    let _guard = tui::enter_tui()?;

    // sort renderers by increasing z-index
    parties.sort_by_key(|party| party.z_index());

    let mut time = std::time::Instant::now();
    loop {
        // bail on any key press
        if event::poll(POLL_TIME)? {
            let event = event::read()?;
            if matches!(event, Event::Key(_)) {
                break;
            }
        }

        let dt = time.elapsed();
        time = std::time::Instant::now();

        // update each party
        let mut all_done = true;
        for party in &mut parties {
            let still_going = party.update(dt);
            all_done = all_done && !still_going;
        }

        // create an output string, render each party to it, and print
        // that to the screen
        let mut buf = String::new();

        execute!(std::io::stdout(), BeginSynchronizedUpdate)?;

        for party in &mut parties {
            party.render(&mut buf);
        }

        std::io::stdout().write_all(buf.as_bytes())?;
        std::io::stdout().flush()?;

        execute!(std::io::stdout(), EndSynchronizedUpdate)?;

        // bail if every party is done animating
        if all_done {
            break;
        }
    }

    Ok(())
}

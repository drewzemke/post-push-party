use std::io::Write;

use crossterm::event::{self, Event};

use super::FullscreenPartyRenderer;
use crate::tui;

// NOTE: this roughly determines the framerate
const POLL_TIME: std::time::Duration = std::time::Duration::from_millis(10);

pub fn run(mut parties: Vec<Box<dyn FullscreenPartyRenderer>>) -> anyhow::Result<()> {
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

        // update each party
        let dt = time.elapsed();
        time = std::time::Instant::now();
        let still_going = parties.iter_mut().any(|p| p.update(dt));

        // create an output string, render each party to it, and print
        // that to the screen
        let mut buf = String::new();
        for party in &mut parties {
            party.render(&mut buf);
        }
        let _ = std::io::stdout().write_all(buf.as_bytes());

        // bail if every party is done animating
        if !still_going {
            break;
        }
    }

    Ok(())
}

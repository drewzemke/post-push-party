use std::io::{Write, stdout};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event},
    execute,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::{renderer::Renderer, sim::Sim};

mod renderer;
mod sim;

const COLORS: [&str; 4] = [
    "\x1b[38;2;240;80;240m",
    "\x1b[38;2;80;240;240m",
    "\x1b[38;2;240;240;240m",
    "\x1b[38;2;240;240;80m",
];

fn main() -> anyhow::Result<()> {
    // start terminal
    let mut stdout = stdout();
    execute!(stdout, Hide, EnterAlternateScreen)?;
    crossterm::terminal::enable_raw_mode()?;
    execute!(stdout, Clear(ClearType::All))?;

    let (cols, rows) = crossterm::terminal::size().unwrap();

    let mut renderer = Renderer::new(rows as usize, cols as usize, &COLORS);
    let mut sim = Sim::new(cols as f64, rows as f64 * 2.);
    let mut time = std::time::Instant::now();

    loop {
        // bail on any key press
        if event::poll(std::time::Duration::from_millis(10))? {
            let event = event::read()?;
            if matches!(event, Event::Key(_)) {
                break;
            }
        }

        // update sim
        let dt = time.elapsed();
        time = std::time::Instant::now();
        sim.update(dt.as_secs_f64());

        // render
        let output = renderer.render(&sim);
        execute!(stdout, MoveTo(0, 0)).unwrap();
        stdout.write_all(output.as_bytes()).unwrap();
    }

    // restore terminal
    execute!(stdout, Show, LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}

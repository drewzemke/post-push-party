use std::io::{Write, stdout};

use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event},
    execute,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use tixel::{BrailleCanvas, Color};

use crate::party::fireworks::sim::Sim;

const POLL_TIME: std::time::Duration = std::time::Duration::from_millis(10);

pub fn run(colors: &[Color]) -> anyhow::Result<()> {
    // start terminal
    let mut stdout = stdout();
    execute!(stdout, Hide, EnterAlternateScreen)?;
    crossterm::terminal::enable_raw_mode()?;
    execute!(stdout, Clear(ClearType::All))?;

    let (cols, rows) = crossterm::terminal::size()?;

    let mut canvas = BrailleCanvas::new((rows as usize, cols as usize), (0, 0));
    let height = canvas.height() as f64;
    let width = canvas.width() as f64;

    let mut sim = Sim::new(width, height);

    let mut time = std::time::Instant::now();

    loop {
        // bail on any key press
        if event::poll(POLL_TIME)? {
            let event = event::read()?;
            if matches!(event, Event::Key(_)) {
                break;
            }
        }

        // update sim
        let dt = time.elapsed();
        time = std::time::Instant::now();
        let has_visible_particles = sim.update(dt.as_secs_f64());

        // render to canvas
        for p in sim.particles() {
            canvas.set_f(p.x, height - p.y, colors[p.color_idx % colors.len()]);
        }

        // render to screen
        let output = canvas.render();
        let _ = stdout.write_all(output.as_bytes());

        // we're done once the screen is empty
        if !has_visible_particles {
            break;
        }
    }

    // restore terminal
    execute!(stdout, Show, LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}

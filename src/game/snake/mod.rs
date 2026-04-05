use std::{
    io::Write,
    time::{Duration, Instant},
};

use anyhow::Result;
use ratatui::crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use tixel::{Color, HalfCellCanvas};

use crate::{
    game::{
        Game,
        snake::state::{Dir, SnakeGame},
    },
    tui::Terminal,
};

mod state;

const TARGET_FRAME_TIME: Duration = Duration::from_millis(50);
const GAME_DIMS: (usize, usize) = (15, 60);

/// Classic snake game in which users earn points as they grow their snake.
pub struct Snake;

impl Game for Snake {
    fn id(&self) -> &'static str {
        "snake"
    }

    fn name(&self) -> &'static str {
        "Snake"
    }

    fn description(&self) -> &'static str {
        "The classic game made popular by late-90s Nokia cell phones. Earn party points as you collect food and grow your snake."
    }

    fn cost(&self) -> u64 {
        25
    }

    fn run(&self, terminal: &mut Terminal) -> Result<()> {
        let mut stdout = std::io::stdout();

        let size = terminal.size()?;
        let rows = size.height as usize;
        let cols = size.width as usize;

        if rows < GAME_DIMS.0 || cols < GAME_DIMS.1 {
            eprintln!(
                "Error: terminal window too small.\nYour terminal window is {rows} rows x {cols} cols.\nMinimum dimensions are {} rows x {} cols.",
                GAME_DIMS.0, GAME_DIMS.1
            );
            std::process::exit(1);
        }

        // center the game area in the screen
        let offset_x = (cols - GAME_DIMS.1) / 2;
        let offset_y = (rows - GAME_DIMS.0) / 2;

        let mut canvas = HalfCellCanvas::new(GAME_DIMS, (offset_y, offset_x));

        let height = canvas.height();
        let width = canvas.width();

        ratatui::crossterm::terminal::enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen, Hide, Clear(ClearType::All))?;

        let mut running = true;
        let mut quitting = false;

        let mut board = SnakeGame::new(width as i64, height as i64);

        loop {
            let frame_start = Instant::now();

            while ratatui::crossterm::event::poll(Duration::ZERO)? {
                if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                    match code {
                        KeyCode::Char('q') => quitting = true,
                        KeyCode::Char(' ') => running = !running,
                        KeyCode::Up => board.turn(Dir::Up),
                        KeyCode::Down => board.turn(Dir::Down),
                        KeyCode::Left => board.turn(Dir::Left),
                        KeyCode::Right => board.turn(Dir::Right),
                        KeyCode::Char('r') => {
                            board.reset();
                            running = true;
                        }
                        _ => {}
                    }
                }
            }

            if quitting {
                break;
            }

            if running {
                if board.is_dead() {
                    running = false;
                } else {
                    // board
                    for y in 0..height {
                        for x in 0..width {
                            canvas.set_color(x, y, Color::new(10, 10, 10));
                        }
                    }

                    // snake
                    for pos in &board.snake {
                        // FIXME: doesn't work
                        let color = if board.is_dead() {
                            Color::new(100, 100, 100)
                        } else {
                            Color::new(140, 240, 140)
                        };
                        canvas.set_color(pos.0 as usize, pos.1 as usize, color);
                    }

                    // food
                    canvas.set_color(
                        board.food.0 as usize,
                        board.food.1 as usize,
                        Color::new(240, 140, 140),
                    );

                    board.advance();
                }
            }

            let elapsed = frame_start.elapsed();
            if elapsed < TARGET_FRAME_TIME {
                let remaining = TARGET_FRAME_TIME - elapsed;
                std::thread::sleep(remaining);
            }

            if running {
                let output = canvas.render();
                let _ = stdout.write_all(output.as_bytes());
                let _ = stdout.flush();
            }
        }

        execute!(stdout, LeaveAlternateScreen, Show)?;
        ratatui::crossterm::terminal::disable_raw_mode()?;

        Ok(())
    }
}

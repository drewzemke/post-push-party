use std::{
    io::Write,
    time::{Duration, Instant},
};

use anyhow::{Result, bail};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent};
use serde::{Deserialize, Serialize};
use tixel::{Color, HalfCellCanvas};

use crate::{
    game::{
        Game, GameWallet,
        snake::state::{Dir, SnakeGame},
    },
    tui::Terminal,
};

mod state;

const TARGET_FRAME_TIME: Duration = Duration::from_millis(50);
const GAME_DIMS: (usize, usize) = (15, 60);

/// Classic snake game in which users earn points as they grow their snake.
pub struct Snake;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
    high_score: u32,
}

impl Game for Snake {
    type State = State;

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

    fn clear_color(&self) -> (u8, u8, u8) {
        (10, 10, 10)
    }

    fn run(&self, terminal: &mut Terminal, wallet: &GameWallet, state: &mut State) -> Result<()> {
        let mut stdout = std::io::stdout();

        let size = terminal.size()?;
        let rows = size.height as usize;
        let cols = size.width as usize;

        // center the game area in the screen
        let offset_x = (cols.saturating_sub(GAME_DIMS.1)) / 2;
        let offset_y = (rows.saturating_sub(GAME_DIMS.0)) / 2;

        let mut canvas = HalfCellCanvas::new(GAME_DIMS, (offset_y, offset_x));

        let height = canvas.height();
        let width = canvas.width();

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

        // save earned points and high score
        let points = board.score();
        wallet.earn(points)?;
        if points > state.high_score {
            state.high_score = points;
        }

        Ok(())
    }
}

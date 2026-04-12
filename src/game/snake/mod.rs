use std::{
    fmt::Write as _,
    io::Write,
    time::{Duration, Instant},
};

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent};
use serde::{Deserialize, Serialize};
use tixel::{Color, HalfCellCanvas, write_bg_color, write_fg_color, write_move_to};

use crate::{
    game::{
        Game, Wallet,
        snake::game::{Dir, SnakeGame},
    },
    tui::Terminal,
};

mod game;

const TARGET_FRAME_TIME: Duration = Duration::from_millis(50);
const GAME_DIMS: (usize, usize) = (15, 60);

const BG_COLOR: Color = Color::new(10, 10, 10);
const BORDER_COLOR: Color = Color::new(200, 200, 200);
const BOARD_COLOR: Color = Color::new(20, 20, 20);
const SNAKE_COLOR: Color = Color::new(140, 240, 140);
const FRUIT_COLOR: Color = Color::new(240, 140, 140);

/// Classic snake game in which users earn points as they grow their snake.
pub struct Snake;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
    high_score: u64,
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
        100
    }

    fn clear_color(&self) -> (u8, u8, u8) {
        (10, 10, 10)
    }

    fn run(
        &self,
        terminal: &mut Terminal,
        wallet: &mut dyn Wallet,
        state: &mut State,
    ) -> Result<()> {
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

        let mut board = SnakeGame::new(width as i64 - 2, height as i64 - 2);
        let high_score_str = format!("High Score: {}", state.high_score);

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
                            canvas.set_color(x, y, BOARD_COLOR);
                        }
                    }

                    // border
                    for y in [0, height - 1] {
                        for x in 0..width {
                            canvas.set_color(x, y, BORDER_COLOR);
                        }
                    }
                    for y in 0..height {
                        for x in [0, width - 1] {
                            canvas.set_color(x, y, BORDER_COLOR);
                        }
                    }

                    // snake
                    for pos in &board.snake {
                        // add 1 to both coordinates to account for the border
                        canvas.set_color(pos.0 as usize + 1, pos.1 as usize + 1, SNAKE_COLOR);
                    }

                    // food
                    // add 1 to both coordinates to account for the border
                    canvas.set_color(
                        board.food.0 as usize + 1,
                        board.food.1 as usize + 1,
                        FRUIT_COLOR,
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
                let mut output = canvas.render();

                // write the score and high score under the game window
                write_bg_color(&mut output, BG_COLOR);
                write_fg_color(&mut output, BORDER_COLOR);

                write_move_to(
                    &mut output,
                    offset_y + (board.height as usize).div_ceil(2) + 1,
                    offset_x + 1,
                );
                let _ = write!(&mut output, "Score: {}", board.score());

                write_move_to(
                    &mut output,
                    offset_y + (board.height as usize).div_ceil(2) + 1,
                    offset_x + board.width as usize - high_score_str.len() + 1,
                );
                let _ = write!(&mut output, "{high_score_str}");

                // output to the screen
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

// scenes:
// - fade in (1 sec)
// - title screen with instructions and controls
//   - get fruit
//   - arrow keys to move
//   - space to pause
//   - any key start
// - game running
// - game paused (just a message at the bottom, saying how to unpause, or maybe also how to quit)
// - (after death) blinking snake or something show you're dead (1-2 sec)
// - game over screen
//   - how many points were earned
//   - new high score?
//   - press any key to go back to party
// - fade out

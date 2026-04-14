use std::{
    io::Write,
    time::{Duration, Instant},
};

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent};
use serde::{Deserialize, Serialize};
use tixel::HalfCellCanvas;

use crate::{
    game::{
        Game, Wallet,
        snake::game::{Dir, SnakeGame},
    },
    tui::Terminal,
};

mod game;
mod render;

const MAX_GAME_TICK_MS: u64 = 200;
const MIN_GAME_TICK_MS: u64 = 30;

const TARGET_FRAME_TIME: Duration = Duration::from_millis(30);
const GAME_DIMS: (usize, usize) = (15, 60);
const FADE_DUR: Duration = Duration::from_millis(500);
const DEAD_DUR: Duration = Duration::from_secs(2);

/// Classic snake game in which users earn points as they grow their snake.
pub struct Snake;

enum Scene {
    FadeIn { since: Instant },
    Title,
    Running { last_tick: Instant },
    Paused,
    Dead { since: Instant },
    GameOver,
    FadeOut { since: Instant },
    Done,
}

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

        let mut game = SnakeGame::new(width as i64 - 2, height as i64 - 2);

        let mut scene = Scene::FadeIn {
            since: Instant::now(),
        };

        let mut prev_disc = std::mem::discriminant(&scene);

        loop {
            let frame_start = Instant::now();

            let key = read_key()?;

            // quit on 'q', don't even check the scene
            if matches!(key, Some(KeyCode::Char('q'))) {
                break;
            }

            scene = update(scene, key, &mut game);

            // do a full redraw if the scene changed
            let disc = std::mem::discriminant(&scene);
            if disc != prev_disc {
                canvas.reset();
            }
            prev_disc = disc;

            if matches!(scene, Scene::Done) {
                break;
            }

            let mut output = String::new();

            render::render(
                &scene,
                &game,
                &mut canvas,
                &mut output,
                state,
                offset_x,
                offset_y,
            );

            let elapsed = frame_start.elapsed();
            if elapsed < TARGET_FRAME_TIME {
                let remaining = TARGET_FRAME_TIME - elapsed;
                std::thread::sleep(remaining);
            }

            // output to the screen
            let _ = stdout.write_all(output.as_bytes());
            let _ = stdout.flush();
        }

        // save earned points and high score
        let points = game.score();
        wallet.earn(points)?;
        if points > state.high_score {
            state.high_score = points;
        }

        Ok(())
    }
}

fn update(scene: Scene, key: Option<KeyCode>, game: &mut SnakeGame) -> Scene {
    let any_key = key.is_some();
    let is_space = matches!(key, Some(KeyCode::Char(' ')));

    match scene {
        Scene::FadeIn { since } if since.elapsed() > FADE_DUR => Scene::Title,
        Scene::Title if any_key => Scene::Running {
            last_tick: Instant::now(),
        },
        Scene::Running { last_tick } => {
            // snake control
            match key {
                Some(KeyCode::Up) => game.turn(Dir::Up),
                Some(KeyCode::Down) => game.turn(Dir::Down),
                Some(KeyCode::Left) => game.turn(Dir::Left),
                Some(KeyCode::Right) => game.turn(Dir::Right),
                _ => {}
            };

            let last_tick = if last_tick.elapsed() >= tick_length(game) {
                game.advance();
                Instant::now()
            } else {
                last_tick
            };

            // control flow
            if game.is_dead() {
                Scene::Dead {
                    since: Instant::now(),
                }
            } else if is_space {
                Scene::Paused
            } else {
                Scene::Running { last_tick }
            }
        }
        Scene::Paused if is_space => Scene::Running {
            last_tick: Instant::now(),
        },
        Scene::Dead { since } if since.elapsed() > DEAD_DUR => Scene::GameOver,
        Scene::GameOver if any_key => Scene::FadeOut {
            since: Instant::now(),
        },
        Scene::FadeOut { since } if since.elapsed() > FADE_DUR => Scene::Done,
        s => s,
    }
}

fn read_key() -> Result<Option<KeyCode>> {
    Ok(
        if ratatui::crossterm::event::poll(Duration::ZERO)?
            && let Event::Key(KeyEvent { code, .. }) = event::read()?
        {
            Some(code)
        } else {
            None
        },
    )
}

/// controls how fast the tick rate increases. higher is slower.
/// (more precisely, this is the value at which the tick rate will have
/// reached halfway between the starting at target tick rates )
const TICK_INCREASE_MODIFIER: f64 = 10.;

fn lerp_f64(a: f64, b: f64, t: f64) -> f64 {
    a + t * (b - a)
}

/// computes target tick time based on score, getting faster as score increases
fn tick_length(game: &SnakeGame) -> Duration {
    let score = game.score() as f64;
    let v = score / (TICK_INCREASE_MODIFIER + score);
    let millis = lerp_f64(MAX_GAME_TICK_MS as f64, MIN_GAME_TICK_MS as f64, v);
    Duration::from_millis(millis as u64)
}

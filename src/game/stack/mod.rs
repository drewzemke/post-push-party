use std::{
    io::Write,
    time::{Duration, Instant},
};

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent};
use serde::{Deserialize, Serialize};

use crate::{
    game::{
        Game, Wallet,
        stack::game::{Input, StackGame},
    },
    tui::Terminal,
};

mod game;
mod render;

const TARGET_FRAME_TIME: Duration = Duration::from_millis(20);
const FADE_DUR: Duration = Duration::from_millis(500);

/// Timing-based stacking game: stop the moving bar to line it up with the one
/// below. Perfect placements build a multiplier.
pub struct Stack;

enum Scene {
    FadeIn { since: Instant },
    Running,
    GameOver,
    FadeOut { since: Instant },
    Done,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
    high_score: u64,
}

impl Game for Stack {
    type State = State;

    fn id(&self) -> &'static str {
        "stack"
    }

    fn name(&self) -> &'static str {
        "Stack"
    }

    fn description(&self) -> &'static str {
        "Stack the moving bars by stopping them at the right time. Perfect placements build a multiplier for more party points!"
    }

    fn cost(&self) -> u64 {
        30
    }

    fn clear_color(&self) -> (u8, u8, u8) {
        render::CLEAR_COLOR
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

        let bounds = game::game_bounds(cols);
        let mut game = StackGame::new(bounds);

        // the row in which the moving bar is placed
        let action_row = rows / 2;

        let mut scene = Scene::FadeIn {
            since: Instant::now(),
        };
        let mut instructions_visible = true;
        let mut last_cut = Instant::now();
        let mut last_frame = Instant::now();

        loop {
            let frame_start = Instant::now();

            let key = read_key()?;

            // quit on 'q'
            if matches!(key, Some(KeyCode::Char('q'))) {
                break;
            }

            let dt = last_frame.elapsed();
            last_frame = Instant::now();

            scene = update(
                scene,
                key,
                &mut game,
                dt,
                &mut instructions_visible,
                &mut last_cut,
            );

            if matches!(scene, Scene::Done) {
                break;
            }

            let mut out = String::new();
            render::render(
                &scene,
                &game,
                &mut out,
                cols,
                rows,
                action_row,
                last_cut,
                instructions_visible,
            );

            let elapsed = frame_start.elapsed();
            if elapsed < TARGET_FRAME_TIME {
                std::thread::sleep(TARGET_FRAME_TIME - elapsed);
            }

            let _ = stdout.write_all(out.as_bytes());
            let _ = stdout.flush();
        }

        // bank the earned points and update the high score
        let points = game.score();
        wallet.earn(points)?;
        if points > state.high_score {
            state.high_score = points;
        }

        Ok(())
    }
}

fn update(
    scene: Scene,
    key: Option<KeyCode>,
    game: &mut StackGame,
    dt: Duration,
    instructions_visible: &mut bool,
    last_cut: &mut Instant,
) -> Scene {
    let any_key = key.is_some();
    let is_cut = matches!(key, Some(KeyCode::Enter | KeyCode::Char(' ')));

    match scene {
        Scene::FadeIn { since } => {
            // the bar keeps moving while we fade in
            game.tick(dt, None);
            if since.elapsed() > FADE_DUR {
                Scene::Running
            } else {
                Scene::FadeIn { since }
            }
        }
        Scene::Running => {
            let input = is_cut.then_some(Input::Cut);
            if game.tick(dt, input).is_some() {
                // a cut happened: hide instructions and restart the ghost fade
                *instructions_visible = false;
                *last_cut = Instant::now();
            }
            if game.is_game_over() {
                Scene::GameOver
            } else {
                Scene::Running
            }
        }
        Scene::GameOver if any_key => Scene::FadeOut {
            since: Instant::now(),
        },
        Scene::FadeOut { since } if since.elapsed() > FADE_DUR => Scene::Done,
        s => s,
    }
}

fn read_key() -> Result<Option<KeyCode>> {
    Ok(
        if event::poll(Duration::ZERO)?
            && let Event::Key(KeyEvent { code, .. }) = event::read()?
        {
            Some(code)
        } else {
            None
        },
    )
}

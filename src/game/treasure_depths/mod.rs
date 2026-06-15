use std::{
    io::Write as _,
    thread,
    time::{Duration, Instant},
};

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent};
use serde::{Deserialize, Serialize};
use tixel::HalfCellCanvas;

use crate::{
    game::{Game, Wallet},
    tui::Terminal,
};

mod game;
mod menu;
mod render;
mod store;

use game::{Game as CoreGame, Input, Loadout};
use menu::Menu;
use render::Renderer;
use store::Store;

/// (cols, rows) in canvas cells
const GAME_DIMS: (usize, usize) = (60, 20);
const TARGET_FRAME_TIME: Duration = Duration::from_millis(20);
const TICK_TIME: Duration = Duration::from_millis(50);
const FADE_DUR: Duration = Duration::from_millis(500);
/// how long the results screen ignores input, so buffered keys can't skip it
const GAME_OVER_DELAY: Duration = Duration::from_millis(750);
const CLEAR_COLOR: (u8, u8, u8) = (10, 10, 10);

const WELCOME_TITLE: &str = "TREASURE DEPTHS";
const WELCOME_BODY: &[&str] = &[
    "Dig down, find treasure, and climb back",
    "to the surface before fuel or time runs out.",
    "",
    "  Arrows  move / dig",
    "   Space  pause     ",
    "",
    "- Digging burns more fuel than moving.        ",
    "- Return to the surface to bank loot & refuel.",
    "- You only keep the money you bank.           ",
];
const WELCOME_FOOTER: &str = "[Enter] continue   [Q] back to Party";

/// A SteamWorld-Dig-style game: spend party points on upgrades, then dig
/// for treasure and climb back out before your fuel or time runs out.
pub struct TreasureDepths;

enum Scene {
    FadeIn { since: Instant },
    Welcome,
    Store,
    Playing { last_tick: Instant, running: bool },
    GameOver { since: Instant },
    FadeOut { since: Instant },
    Done,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
    best_haul: u64,
}

impl Game for TreasureDepths {
    type State = State;

    fn id(&self) -> &'static str {
        "treasure_depths"
    }

    fn name(&self) -> &'static str {
        "Treasure Depths"
    }

    fn description(&self) -> &'static str {
        "Dig deep for treasure and climb back to the surface with your haul before your fuel or time runs out."
    }

    fn cost(&self) -> u64 {
        150
    }

    fn clear_color(&self) -> (u8, u8, u8) {
        CLEAR_COLOR
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

        // center the game area; keep a row above for the top hud
        let offset_x = cols.saturating_sub(GAME_DIMS.0) / 2;
        let offset_y = (rows.saturating_sub(GAME_DIMS.1) / 2).max(1);
        let offset = (offset_x, offset_y);

        let mut canvas = HalfCellCanvas::new(GAME_DIMS, (offset_x, offset_y));
        let mut renderer = Renderer::new();
        let menu = Menu::new((cols, rows));

        // the store is capped at the player's current point balance
        let mut store = Store::new(wallet.balance()?);
        // placeholder world shown behind the welcome/store modals; replaced with
        // the real run once the player buys upgrades and starts
        let mut game = CoreGame::new(canvas.width(), Loadout::base());

        let mut scene = Scene::FadeIn {
            since: Instant::now(),
        };
        let mut prev_disc = std::mem::discriminant(&scene);

        loop {
            let frame_start = Instant::now();

            // drain every key queued this frame so input can't back up and lag
            // behind the player (one event per frame causes "stuck" controls)
            let mut quit = false;
            while let Some(code) = read_key()? {
                // 'q' always bails back to Party
                if matches!(code, KeyCode::Char('q' | 'Q')) {
                    quit = true;
                    break;
                }

                scene = match scene {
                    Scene::Welcome => {
                        if code == KeyCode::Enter {
                            Scene::Store
                        } else {
                            Scene::Welcome
                        }
                    }
                    Scene::Store => match code {
                        KeyCode::Up => {
                            store.select_prev();
                            Scene::Store
                        }
                        KeyCode::Down => {
                            store.select_next();
                            Scene::Store
                        }
                        KeyCode::Left => {
                            store.tier_down();
                            Scene::Store
                        }
                        KeyCode::Right => {
                            store.tier_up();
                            Scene::Store
                        }
                        KeyCode::Enter => {
                            // commit the purchase and start the run
                            wallet.spend(store.spent())?;
                            game = CoreGame::new(canvas.width(), store.loadout());
                            Scene::Playing {
                                last_tick: Instant::now(),
                                running: true,
                            }
                        }
                        _ => Scene::Store,
                    },
                    Scene::Playing {
                        mut last_tick,
                        mut running,
                    } => {
                        match code {
                            KeyCode::Char(' ') => {
                                running = !running;
                                // reset the tick clock on resume so a long pause
                                // doesn't burn time/fuel in one big catch-up tick
                                if running {
                                    last_tick = Instant::now();
                                }
                            }
                            KeyCode::Up => game.handle(Input::Up),
                            KeyCode::Down => game.handle(Input::Down),
                            KeyCode::Left => game.handle(Input::Left),
                            KeyCode::Right => game.handle(Input::Right),
                            _ => {}
                        }
                        Scene::Playing { last_tick, running }
                    }
                    Scene::GameOver { since } => {
                        // ignore buffered input briefly so results aren't skipped
                        if since.elapsed() > GAME_OVER_DELAY {
                            Scene::FadeOut {
                                since: Instant::now(),
                            }
                        } else {
                            Scene::GameOver { since }
                        }
                    }
                    // fades and Done ignore input
                    other => other,
                };
            }

            if quit {
                break;
            }

            // time-based progression, applied once per frame
            scene = match scene {
                Scene::FadeIn { since } => {
                    if since.elapsed() > FADE_DUR {
                        Scene::Welcome
                    } else {
                        Scene::FadeIn { since }
                    }
                }
                Scene::Playing {
                    mut last_tick,
                    running,
                } => {
                    if running && last_tick.elapsed() > TICK_TIME {
                        game.tick(last_tick.elapsed());
                        last_tick = Instant::now();
                    }

                    if game.is_over() {
                        Scene::GameOver {
                            since: Instant::now(),
                        }
                    } else {
                        Scene::Playing { last_tick, running }
                    }
                }
                Scene::FadeOut { since } => {
                    if since.elapsed() > FADE_DUR {
                        Scene::Done
                    } else {
                        Scene::FadeOut { since }
                    }
                }
                other => other,
            };

            // force a full redraw whenever the scene changes
            let disc = std::mem::discriminant(&scene);
            if disc != prev_disc {
                canvas.reset();
            }
            prev_disc = disc;

            if matches!(scene, Scene::Done) {
                break;
            }

            let mut output = String::new();
            render_scene(
                &scene,
                &mut renderer,
                &mut canvas,
                &mut game,
                &store,
                &menu,
                &mut output,
                offset,
                (cols, rows),
                state,
            );

            let elapsed = frame_start.elapsed();
            if elapsed < TARGET_FRAME_TIME {
                thread::sleep(TARGET_FRAME_TIME - elapsed);
            }

            let _ = stdout.write_all(output.as_bytes());
            let _ = stdout.flush();
        }

        // banked treasure converts to party points 1:1 and is kept even if the
        // player quits mid-run (banked loot is already secured)
        let haul = game.bank_value();
        if haul > 0 {
            wallet.earn(haul)?;
        }
        state.best_haul = state.best_haul.max(haul);

        Ok(())
    }
}

#[expect(clippy::too_many_arguments)]
fn render_scene(
    scene: &Scene,
    renderer: &mut Renderer,
    canvas: &mut HalfCellCanvas,
    game: &mut CoreGame,
    store: &Store,
    menu: &Menu,
    output: &mut String,
    offset: (usize, usize),
    term: (usize, usize),
    state: &State,
) {
    match scene {
        Scene::FadeIn { since } => {
            let opacity = (since.elapsed().as_secs_f64() / FADE_DUR.as_secs_f64()).min(1.);
            renderer.render(canvas, game, output, offset, false, opacity, CLEAR_COLOR);
        }
        Scene::Welcome => {
            renderer.render(canvas, game, output, offset, false, 1., CLEAR_COLOR);
            menu.render(output, WELCOME_TITLE, WELCOME_BODY, WELCOME_FOOTER);
        }
        Scene::Store => {
            renderer.render(canvas, game, output, offset, false, 1., CLEAR_COLOR);
            store.render(output, term);
        }
        Scene::Playing { .. } => {
            renderer.render(canvas, game, output, offset, true, 1., CLEAR_COLOR);
        }
        Scene::GameOver { .. } => {
            renderer.render(canvas, game, output, offset, true, 1., CLEAR_COLOR);

            let haul = game.bank_value();
            let new_best = haul > state.best_haul;

            let depth = format!("Max depth        \u{2193}{}", game.max_depth());
            let banked = format!("Gained           {haul} P");
            let best = if new_best {
                "\u{2605} New best haul! \u{2605}".to_string()
            } else {
                format!("Best haul        {} P", state.best_haul)
            };

            menu.render(
                output,
                "RUN OVER",
                &[&depth, &banked, &best],
                "[any key] return to Party",
            );
        }
        Scene::FadeOut { since } => {
            let opacity = (1. - since.elapsed().as_secs_f64() / FADE_DUR.as_secs_f64()).max(0.);
            renderer.render(canvas, game, output, offset, false, opacity, CLEAR_COLOR);
        }
        Scene::Done => {}
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

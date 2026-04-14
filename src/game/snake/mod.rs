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

const MAX_GAME_TICK_MS: u64 = 200;
const MIN_GAME_TICK_MS: u64 = 30;

const TARGET_FRAME_TIME: Duration = Duration::from_millis(30);
const GAME_DIMS: (usize, usize) = (15, 60);
const FADE_DUR: Duration = Duration::from_millis(500);
const DEAD_DUR: Duration = Duration::from_secs(2);

const BG_COLOR_U8: (u8, u8, u8) = (10, 10, 10);
const BG_COLOR: Color = Color::new(BG_COLOR_U8.0, BG_COLOR_U8.1, BG_COLOR_U8.2);
const FG_COLOR_U8: (u8, u8, u8) = (200, 200, 200);
const FG_COLOR: Color = Color::new(FG_COLOR_U8.0, FG_COLOR_U8.1, FG_COLOR_U8.2);

const BOARD_COLOR_U8: (u8, u8, u8) = (20, 20, 20);
const BOARD_COLOR: Color = Color::new(20, 20, 20);
const SNAKE_COLOR: Color = Color::new(140, 240, 140);
const FRUIT_COLOR: Color = Color::new(240, 140, 140);

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

            render(
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

fn render(
    scene: &Scene,
    game: &SnakeGame,
    canvas: &mut HalfCellCanvas,
    output: &mut String,
    state: &State,
    offset_x: usize,
    offset_y: usize,
) {
    match scene {
        Scene::FadeIn { since } => {
            let opacity = since.elapsed().as_secs_f64() / FADE_DUR.as_secs_f64();
            render_board_with_opacity(canvas, opacity);
            render_border_with_opacity(canvas, opacity);
            *output = canvas.render();
        }
        Scene::Title => {
            render_board(canvas);
            render_border(canvas);
            *output = canvas.render();
            render_title(output, canvas.width(), offset_x, offset_y);
        }
        Scene::Running { .. } => {
            render_board(canvas);
            render_border(canvas);
            render_snake(canvas, game);
            render_food(canvas, game);
            *output = canvas.render();
            render_score_line(output, game, state, offset_x, offset_y);
        }
        Scene::Paused => render_paused_line(output, game, offset_x, offset_y),
        Scene::Dead { .. } => clear_message_line(output, game, offset_x, offset_y),
        Scene::GameOver => {
            render_board(canvas);
            render_border(canvas);
            *output = canvas.render();
            render_game_over(
                output,
                canvas.width(),
                offset_x,
                offset_y,
                game.score(),
                game.score() > state.high_score,
            );
        }
        Scene::FadeOut { since } => {
            let opacity = 1. - since.elapsed().as_secs_f64() / FADE_DUR.as_secs_f64();
            render_board_with_opacity(canvas, opacity);
            render_border_with_opacity(canvas, opacity);
            *output = canvas.render();
        }
        Scene::Done => {}
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

fn render_border(canvas: &mut HalfCellCanvas) {
    render_border_with_opacity(canvas, 1.);
}

fn render_border_with_opacity(canvas: &mut HalfCellCanvas, opacity: f64) {
    let color = lerp_color(FG_COLOR_U8, BG_COLOR_U8, opacity);
    for y in [0, canvas.height() - 1] {
        for x in 0..canvas.width() {
            canvas.set_color(x, y, color);
        }
    }
    for y in 0..canvas.height() {
        for x in [0, canvas.width() - 1] {
            canvas.set_color(x, y, color);
        }
    }
}

fn render_board(canvas: &mut HalfCellCanvas) {
    render_board_with_opacity(canvas, 1.);
}
fn render_board_with_opacity(canvas: &mut HalfCellCanvas, opacity: f64) {
    let color = lerp_color(BOARD_COLOR_U8, BG_COLOR_U8, opacity);

    for y in 0..canvas.height() {
        for x in 0..canvas.width() {
            canvas.set_color(x, y, color);
        }
    }
}

fn lerp_u8(v: u8, bg: u8, opacity: f64) -> u8 {
    (bg as f64 * (1. - opacity)) as u8 + (v as f64 * opacity) as u8
}

fn lerp_color(color: (u8, u8, u8), bg_color: (u8, u8, u8), opacity: f64) -> Color {
    let r = lerp_u8(color.0, bg_color.0, opacity);
    let g = lerp_u8(color.1, bg_color.1, opacity);
    let b = lerp_u8(color.2, bg_color.2, opacity);
    Color::new(r, g, b)
}

fn render_food(canvas: &mut HalfCellCanvas, game: &SnakeGame) {
    // add 1 to both coordinates to account for the border
    canvas.set_color(
        game.food.0 as usize + 1,
        game.food.1 as usize + 1,
        FRUIT_COLOR,
    );
}

fn render_snake(canvas: &mut HalfCellCanvas, game: &SnakeGame) {
    for pos in &game.snake {
        // add 1 to both coordinates to account for the border
        canvas.set_color(pos.0 as usize + 1, pos.1 as usize + 1, SNAKE_COLOR);
    }
}

/// write the score and high score under the game window
fn render_score_line(
    out: &mut String,
    game: &SnakeGame,
    state: &State,
    offset_x: usize,
    offset_y: usize,
) {
    clear_message_line(out, game, offset_x, offset_y);
    write_bg_color(out, BG_COLOR);
    write_fg_color(out, FG_COLOR);

    write_move_to(
        out,
        offset_y + (game.height as usize).div_ceil(2) + 1,
        offset_x + 1,
    );
    let _ = write!(out, "Score: {}", game.score());

    let high_score_str = format!("High Score: {}", state.high_score);
    write_move_to(
        out,
        offset_y + (game.height as usize).div_ceil(2) + 1,
        offset_x + game.width as usize - high_score_str.len() + 1,
    );
    let _ = write!(out, "{high_score_str}");
}

/// clears the line under the game board
fn clear_message_line(out: &mut String, game: &SnakeGame, offset_x: usize, offset_y: usize) {
    write_bg_color(out, BG_COLOR);
    write_fg_color(out, FG_COLOR);

    write_move_to(
        out,
        offset_y + (game.height as usize).div_ceil(2) + 1,
        offset_x + 1,
    );
    let _ = write!(out, "{}", " ".repeat(game.width as usize));
}

/// writes a paused message to the screen
fn render_paused_line(out: &mut String, game: &SnakeGame, offset_x: usize, offset_y: usize) {
    clear_message_line(out, game, offset_x, offset_y);
    write_bg_color(out, BG_COLOR);
    write_fg_color(out, FG_COLOR);

    render_centered(
        out,
        "PAUSED  (press space to resume)",
        offset_y + (game.height as usize).div_ceil(2) + 1,
        offset_x,
        game.width as usize,
    );
}

fn render_centered(out: &mut String, text: &str, row: usize, offset_x: usize, width: usize) {
    let col = offset_x + (width - text.len()) / 2;
    write_move_to(out, row, col);
    let _ = write!(out, "{text}");
}

fn render_title(out: &mut String, width: usize, offset_x: usize, offset_y: usize) {
    write_fg_color(out, FG_COLOR);
    write_bg_color(out, BOARD_COLOR);

    render_centered(out, "Welcome to Snake!", offset_y + 3, offset_x, width);

    render_centered(out, "Collect fruit.", offset_y + 5, offset_x, width);
    render_centered(
        out,
        "Don't run into yourself or the walls.",
        offset_y + 6,
        offset_x,
        width,
    );

    render_centered(
        out,
        "Use the arrow keys to move.",
        offset_y + 8,
        offset_x,
        width,
    );
    render_centered(
        out,
        "Press space to pause, q to return to Party.",
        offset_y + 9,
        offset_x,
        width,
    );
    render_centered(
        out,
        "Press any key to play!",
        offset_y + 11,
        offset_x,
        width,
    );
}

fn render_game_over(
    out: &mut String,
    width: usize,
    offset_x: usize,
    offset_y: usize,
    score: u64,
    new_high_score: bool,
) {
    write_fg_color(out, FG_COLOR);
    write_bg_color(out, BOARD_COLOR);

    render_centered(out, "Game Over!", offset_y + 3, offset_x, width);

    let fruit_word = if score == 1 { "fruit" } else { "fruits" };
    render_centered(
        out,
        &format!("You collected {score} {fruit_word}."),
        offset_y + 5,
        offset_x,
        width,
    );
    if new_high_score {
        render_centered(
            out,
            "(That's a new high score!)",
            offset_y + 6,
            offset_x,
            width,
        );
    }

    let points_word = if score == 1 { "point" } else { "points" };
    render_centered(
        out,
        &format!("You earned {score} party {points_word}!"),
        offset_y + 9,
        offset_x,
        width,
    );
    render_centered(
        out,
        "Press any key to return to Party.",
        offset_y + 10,
        offset_x,
        width,
    );
}

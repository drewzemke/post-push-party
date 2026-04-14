use std::fmt::Write as _;
use tixel::{Color, HalfCellCanvas, write_bg_color, write_fg_color, write_move_to};

use super::{FADE_DUR, Scene, SnakeGame, State};

const BG_COLOR_U8: (u8, u8, u8) = (10, 10, 10);
const BG_COLOR: Color = Color::new(BG_COLOR_U8.0, BG_COLOR_U8.1, BG_COLOR_U8.2);
const FG_COLOR_U8: (u8, u8, u8) = (200, 200, 200);
const FG_COLOR: Color = Color::new(FG_COLOR_U8.0, FG_COLOR_U8.1, FG_COLOR_U8.2);

const BOARD_COLOR_U8: (u8, u8, u8) = (20, 20, 20);
const BOARD_COLOR: Color = Color::new(20, 20, 20);
const SNAKE_COLOR: Color = Color::new(140, 240, 140);
const FRUIT_COLOR: Color = Color::new(240, 140, 140);

pub fn render(
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

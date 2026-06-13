use std::fmt::Write as _;
use std::time::{Duration, Instant};

use tixel::{
    Color,
    utils::{write_bg_color, write_fg_color, write_move_to},
};

use super::game::StackGame;

/// color the terminal transitions to/from when entering and leaving the game
pub const CLEAR_COLOR: (u8, u8, u8) = (10, 10, 10);

const BAR_SAT: f64 = 0.9;
const BAR_LUM: f64 = 0.7;

const GHOST_SAT_FACTOR: f64 = 0.6;
const GHOST_LUM_FACTOR: f64 = 0.6;
pub const GHOST_FADE_TIME: Duration = Duration::from_secs(1);

const PERFECT_TEXT_FADE: Duration = Duration::from_secs(2);

const BORDER_LINE_COLOR: (u8, u8, u8) = (130, 130, 130);
const TEXT_COLOR: (u8, u8, u8) = (255, 255, 255);

const INSTRUCTIONS: &str = "Press space or enter to stop the bar, q to quit.";

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let h = h.rem_euclid(360.);
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h2 = h / 60.0;
    let x = c * (1.0 - (h2 % 2.0 - 1.0).abs());
    let (r1, g1, b1) = match h2 as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    (
        ((r1 + m) * 255.0) as u8,
        ((g1 + m) * 255.0) as u8,
        ((b1 + m) * 255.0) as u8,
    )
}

fn lerp_f64(a: f64, b: f64, t: f64) -> f64 {
    a + t * (b - a)
}

fn lerp_u8(a: u8, b: u8, t: f64) -> u8 {
    lerp_f64(a as f64, b as f64, t) as u8
}

fn lerp_rgb(a: (u8, u8, u8), b: (u8, u8, u8), t: f64) -> (u8, u8, u8) {
    (
        lerp_u8(a.0, b.0, t),
        lerp_u8(a.1, b.1, t),
        lerp_u8(a.2, b.2, t),
    )
}

/// fades a color toward the clear color. fade of 1 is full color, 0 is clear color
fn fade_color(color: (u8, u8, u8), fade: f64) -> Color {
    Color::from(lerp_rgb(CLEAR_COLOR, color, fade.clamp(0., 1.)))
}

fn get_bg_color(stack: usize, row: usize, rows: usize) -> (u8, u8, u8) {
    let shade = stack + (rows - row);
    let shade = if shade > 255 { 255 } else { shade as u8 };
    (shade, shade, shade)
}

fn render_segment(
    out: &mut String,
    row: usize,
    left: f64,
    right: f64,
    fg: Color,
    left_bg: Color,
    right_bg: Color,
) {
    let left_q = (left * 2.).floor() / 2.;
    let right_q = (right * 2.).floor() / 2.;

    if right_q <= left_q {
        return;
    }

    let left_col = left_q as usize;
    let right_col = right_q as usize;
    let fract_l = left_q.fract();
    let fract_r = right_q.fract();

    write_move_to(out, row, left_col);
    write_fg_color(out, fg);

    if fract_l > 0. {
        write_bg_color(out, left_bg);
        let _ = write!(out, "▐");
    }

    let int_start = if fract_l > 0. { left_col + 1 } else { left_col };
    let int_width = right_col.saturating_sub(int_start);
    if int_width > 0 {
        write_bg_color(out, left_bg);
        let _ = write!(out, "{}", "█".repeat(int_width));
    }

    if fract_r > 0. {
        write_bg_color(out, right_bg);
        let _ = write!(out, "▌");
    }
}

fn render_bg_row(
    out: &mut String,
    row: usize,
    cols: usize,
    bounds: (usize, usize),
    color: (u8, u8, u8),
    fade: f64,
) {
    write_move_to(out, row, 0);
    write_bg_color(out, fade_color(color, fade));
    write_fg_color(out, fade_color(BORDER_LINE_COLOR, fade));
    let to_first_border = bounds.0 - 1;
    let to_second_border = bounds.1 - to_first_border - 2;
    let to_right_edge = cols - bounds.1;
    let _ = write!(
        out,
        "{}┆{}┆{}",
        " ".repeat(to_first_border),
        " ".repeat(to_second_border),
        " ".repeat(to_right_edge),
    );
}

#[expect(clippy::too_many_arguments)]
fn render_bar(
    out: &mut String,
    bar: &super::game::Bar,
    row: usize,
    num_cols: usize,
    bg_color: (u8, u8, u8),
    ghost_factor: f64,
    flash: f64,
    fade: f64,
) {
    let bar_color = hsl_to_rgb(bar.hue, BAR_SAT, BAR_LUM);
    let bar_color = lerp_rgb(bar_color, (255, 255, 255), flash);

    render_segment(
        out,
        row,
        bar.quantized_left(),
        bar.quantized_right(),
        fade_color(bar_color, fade),
        fade_color(bg_color, fade),
        fade_color(bg_color, fade),
    );

    if bar.deleted.abs() > 0.01 {
        let ghost_color = hsl_to_rgb(
            bar.hue,
            BAR_SAT * GHOST_SAT_FACTOR,
            BAR_LUM * GHOST_LUM_FACTOR,
        );
        let ghost_color = lerp_rgb(bg_color, ghost_color, ghost_factor);

        let q_left = bar.quantized_left();
        let q_right = bar.quantized_right();

        let (g_left, g_right) = if bar.deleted < 0. {
            ((q_left + bar.deleted).max(0.), q_left)
        } else {
            (q_right, (q_right + bar.deleted).min(num_cols as f64))
        };

        let left_bg = if bar.deleted > 0. {
            bar_color
        } else {
            bg_color
        };
        let right_bg = if bar.deleted < 0. {
            bar_color
        } else {
            bg_color
        };

        render_segment(
            out,
            row,
            g_left,
            g_right,
            fade_color(ghost_color, fade),
            fade_color(left_bg, fade),
            fade_color(right_bg, fade),
        );
    }
}

fn write_centered(
    out: &mut String,
    row: usize,
    cols: usize,
    text: &str,
    fg: (u8, u8, u8),
    bg: (u8, u8, u8),
    fade: f64,
) {
    let col = cols.saturating_sub(text.chars().count()) / 2;
    write_move_to(out, row, col);
    write_bg_color(out, fade_color(bg, fade));
    write_fg_color(out, fade_color(fg, fade));
    let _ = write!(out, "{text}");
}

/// draws the whole playfield: background, the moving bar, the stack, and the
/// default bars filling the bottom of the screen
fn render_game(
    out: &mut String,
    game: &StackGame,
    cols: usize,
    rows: usize,
    action_row: usize,
    last_cut: Instant,
    fade: f64,
) {
    let stack_len = game.stack.len();
    let bounds = super::game::game_bounds(cols);

    // background
    for row in 0..rows {
        let color = get_bg_color(stack_len, row, rows);
        render_bg_row(out, row, cols, bounds, color, fade);
    }

    // moving bar
    let bg_color = get_bg_color(stack_len, action_row, rows);
    render_bar(out, &game.current, action_row, cols, bg_color, 0., 0., fade);

    // bars from the stack
    for (idx, bar) in game.stack.iter().rev().enumerate() {
        let row = action_row + 1 + idx;
        if row >= rows {
            break;
        }

        let bg_color = get_bg_color(stack_len, row, rows);
        let ghost_factor = if idx == 0 && !game.is_game_over() {
            GHOST_FADE_TIME
                .saturating_sub(last_cut.elapsed())
                .as_secs_f64()
                / GHOST_FADE_TIME.as_secs_f64()
        } else {
            0.
        };
        let flash = if idx == 0 { game.flash_factor() } else { 0. };

        render_bar(out, bar, row, cols, bg_color, ghost_factor, flash, fade);
    }

    // default bars all the way down
    for row in (action_row + stack_len + 1)..rows {
        let bg_color = get_bg_color(stack_len, row, rows);
        let offset = row - (action_row + stack_len);
        let bar = game.default_bar(offset);
        render_bar(out, &bar, row, cols, bg_color, 0., 0., fade);
    }
}

/// the "perfect!" text that fades out next to the top bar after a perfect cut
fn render_perfect_text(out: &mut String, game: &StackGame, action_row: usize, rows: usize) {
    let Some(bar) = game.stack.last() else {
        return;
    };
    let row = action_row + 1;
    if row >= rows {
        return;
    }
    let Some(t) = game.perfect_cut_at else {
        return;
    };
    let elapsed = t.elapsed();
    if elapsed >= PERFECT_TEXT_FADE {
        return;
    }

    let fade = 1. - elapsed.as_secs_f64() / PERFECT_TEXT_FADE.as_secs_f64();
    let bg_color = get_bg_color(game.stack.len(), row, rows);
    let text_color = lerp_rgb(bg_color, TEXT_COLOR, fade);
    let col = bar.quantized_right() as usize + 1;

    write_move_to(out, row, col);
    write_fg_color(out, Color::from(text_color));
    write_bg_color(out, Color::from(bg_color));
    let run_text = if game.perfect_run > 1 {
        format!(" x{}", game.perfect_run)
    } else {
        String::new()
    };
    let _ = write!(out, "perfect{run_text}!");
}

fn render_top_line(out: &mut String, game: &StackGame, cols: usize, rows: usize, fade: f64) {
    let bg = get_bg_color(game.stack.len(), 1, rows);
    write_centered(out, 1, cols, INSTRUCTIONS, TEXT_COLOR, bg, fade);
}

fn render_score_line(out: &mut String, game: &StackGame, rows: usize) {
    let bg = get_bg_color(game.stack.len(), 1, rows);
    write_move_to(out, 1, 2);
    write_bg_color(out, Color::from(bg));
    write_fg_color(out, Color::from(TEXT_COLOR));
    let _ = write!(
        out,
        "stack: {}  mult: {:.1}",
        game.raw_points(),
        game.multiplier()
    );
}

fn render_game_over(out: &mut String, game: &StackGame, cols: usize, rows: usize) {
    let points = game.score();
    let bg5 = get_bg_color(game.stack.len(), 5, rows);
    let bg7 = get_bg_color(game.stack.len(), 7, rows);
    let bg9 = get_bg_color(game.stack.len(), 9, rows);

    write_centered(out, 5, cols, "Game Over", TEXT_COLOR, bg5, 1.);
    let earned = format!("You earned {points} party points!");
    write_centered(out, 7, cols, &earned, TEXT_COLOR, bg7, 1.);
    write_centered(
        out,
        9,
        cols,
        "Press any key to return to party.",
        TEXT_COLOR,
        bg9,
        1.,
    );
}

#[allow(clippy::too_many_arguments)]
pub fn render(
    scene: &super::Scene,
    game: &StackGame,
    out: &mut String,
    cols: usize,
    rows: usize,
    action_row: usize,
    last_cut: Instant,
    instructions_visible: bool,
) {
    use super::Scene;
    match scene {
        Scene::FadeIn { since } => {
            let fade = since.elapsed().as_secs_f64() / super::FADE_DUR.as_secs_f64();
            render_game(out, game, cols, rows, action_row, last_cut, fade);
            if instructions_visible {
                render_top_line(out, game, cols, rows, fade);
            }
        }
        Scene::Running => {
            render_game(out, game, cols, rows, action_row, last_cut, 1.);
            if instructions_visible {
                render_top_line(out, game, cols, rows, 1.);
            } else {
                render_score_line(out, game, rows);
            }
            render_perfect_text(out, game, action_row, rows);
        }
        Scene::GameOver => {
            render_game(out, game, cols, rows, action_row, last_cut, 1.);
            render_game_over(out, game, cols, rows);
        }
        Scene::FadeOut { since } => {
            let fade = 1. - since.elapsed().as_secs_f64() / super::FADE_DUR.as_secs_f64();
            render_game(out, game, cols, rows, action_row, last_cut, fade);
        }
        Scene::Done => {}
    }
}

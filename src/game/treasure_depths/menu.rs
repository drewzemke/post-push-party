use std::fmt::Write as _;

use tixel::{
    Color,
    utils::{write_bg_color, write_fg_color, write_move_to},
};

const MIN_INNER_WIDTH: usize = 40;
const H_PADDING: usize = 3;

pub const BG_COLOR: Color = Color::Rgb(0, 0, 0);
pub const BORDER_COLOR: Color = Color::Rgb(210, 210, 210);
const TITLE_COLOR: Color = Color::Rgb(250, 230, 120);
const TEXT_COLOR: Color = Color::Rgb(220, 220, 220);
const FOOTER_COLOR: Color = Color::Rgb(140, 140, 140);

/// draws an empty bordered box and fills its interior with the background
/// color. callers write their content on top afterward (in the same buffer,
/// so the background color persists).
pub fn draw_box(buf: &mut String, offset: (usize, usize), inner_width: usize, inner_height: usize) {
    write_bg_color(buf, BG_COLOR);
    write_fg_color(buf, BORDER_COLOR);

    write_move_to(buf, offset.0, offset.1);
    let _ = write!(buf, "▛{}▜", "▀".repeat(inner_width));

    for row in 1..=inner_height {
        write_move_to(buf, offset.0, offset.1 + row);
        let _ = write!(buf, "▌{}▐", " ".repeat(inner_width));
    }

    write_move_to(buf, offset.0, offset.1 + inner_height + 1);
    let _ = write!(buf, "▙{}▟", "▄".repeat(inner_width));
}

/// writes text at a terminal position in the given color; assumes the
/// background color was already set (e.g. by `draw_box`)
pub fn write_text(buf: &mut String, row: usize, col: usize, text: &str, color: Color) {
    write_move_to(buf, col, row);
    write_fg_color(buf, color);
    let _ = write!(buf, "{text}");
}

/// top-left offset that centers a box of the given size on screen
pub fn center(term: (usize, usize), width: usize, height: usize) -> (usize, usize) {
    (
        term.0.saturating_sub(width) / 2,
        term.1.saturating_sub(height) / 2,
    )
}

/// a centered, non-interactive modal panel drawn over the game
pub struct Menu {
    /// terminal size in (cols, rows)
    term: (usize, usize),
}

impl Menu {
    pub fn new(term: (usize, usize)) -> Self {
        Self { term }
    }

    /// draws a bordered box centered on screen with a title, body lines,
    /// and a footer hint, all horizontally centered
    pub fn render(&self, buf: &mut String, title: &str, body: &[&str], footer: &str) {
        // content rows: title, blank, body..., blank, footer
        let mut rows: Vec<(&str, Color)> = Vec::with_capacity(body.len() + 4);
        rows.push((title, TITLE_COLOR));
        rows.push(("", TEXT_COLOR));
        for &line in body {
            rows.push((line, TEXT_COLOR));
        }
        rows.push(("", TEXT_COLOR));
        rows.push((footer, FOOTER_COLOR));

        let widest = rows
            .iter()
            .map(|(t, _)| t.chars().count())
            .max()
            .unwrap_or(0);
        let inner_width = (widest + 2 * H_PADDING).max(MIN_INNER_WIDTH);
        let inner_height = rows.len();

        let offset = center(self.term, inner_width + 2, inner_height + 2);

        draw_box(buf, offset, inner_width, inner_height);

        for (i, (text, color)) in rows.iter().enumerate() {
            let len = text.chars().count();
            let col = offset.0 + 1 + inner_width.saturating_sub(len) / 2;
            write_text(buf, offset.1 + 1 + i, col, text, *color);
        }
    }
}

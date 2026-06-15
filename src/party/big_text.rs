use std::{fmt::Write as _, time::Duration};

use tixel::{
    Color,
    utils::{write_fg_color, write_fg_reset, write_move_to},
};

use crate::party::{FullscreenPartyRenderer, PartyEntry, PartyInfo, PartyRenderer};

use super::{Palette, random_pick};

const WORDS: &[&str] = &["NICE!", "SWEET!", "SICK!", "DOPE!", "COOL!", "YEAH!"];

const LETTER_HEIGHT: usize = 6;

/// Prints a random word in large ASCII art letters.
pub static BIG_TEXT_PARTY: PartyEntry = PartyEntry {
    info: PartyInfo {
        id: "big_text",
        name: "Big Text Party",
        description: "Shows a random word in big ASCII art letters.",
        cost: 2000,
        supports_color: true,
    },
    renderer: PartyRenderer::Fullscreen {
        create: BigTextRenderer::create,
    },
};

struct BigTextRenderer {
    /// (width, height)
    text_dims: (usize, usize),

    /// (cols, rows)
    offset: (usize, usize),

    /// the big text itself, without color data, split into lines
    chars: Vec<Vec<char>>,

    /// determines which characters in the `word_lines` array are visible
    mask: Vec<Vec<bool>>,

    /// determins the color of individual columns
    colors: Vec<Color>,

    /// tracks how long the animation has been running
    elapsed: Duration,
}

impl BigTextRenderer {
    pub fn create(
        width: u16,
        height: u16,
        palette: &'static Palette,
    ) -> Box<dyn FullscreenPartyRenderer> {
        // choose a word
        let word = random_pick(WORDS);

        // gather lines by scanning through each letter
        let mut chars = vec![vec![]; LETTER_HEIGHT];
        let mut colors = vec![];

        for (idx, ch) in word.chars().enumerate() {
            let color = palette.get_color(idx);
            if let Some(letter) = get_letter(ch) {
                for (i, letter_line) in letter.iter().enumerate() {
                    chars[i].extend(letter_line.chars());
                    if idx < word.len() - 1 {
                        chars[i].push(' ');
                    }
                }
                colors.extend(vec![color; letter[0].chars().count() + 1]);
            }
        }

        let text_width = chars[0].len();
        let text_height = chars.len();

        let offset_x = (width as usize - text_width) / 2;
        let offset_y = (height as usize - text_height) / 2;

        let mask = vec![vec![false; text_width]; text_height];

        Box::new(Self {
            text_dims: (text_width, text_height),
            offset: (offset_x, offset_y),
            chars,
            mask,
            colors,
            elapsed: Duration::ZERO,
        })
    }
}

const ANIMATE_START_TIME: Duration = Duration::from_millis(500);
const TOTAL_TIME: Duration = Duration::from_secs(5);
const REVEAL_RATE: f64 = 40.;

impl FullscreenPartyRenderer for BigTextRenderer {
    fn z_index(&self) -> u32 {
        10
    }

    fn update(&mut self, dt: std::time::Duration) -> bool {
        // update the timer
        self.elapsed += dt;
        if self.elapsed > TOTAL_TIME {
            return false;
        }

        // update the mask
        for (row_idx, row) in self.mask.iter_mut().enumerate() {
            for (col_idx, b) in row.iter_mut().enumerate() {
                // TODO: add different ways of animating in?
                *b = (col_idx as f64 + 2. * row_idx as f64) / REVEAL_RATE
                    + ANIMATE_START_TIME.as_secs_f64()
                    < self.elapsed.as_secs_f64();
            }
        }

        true
    }

    fn render(&mut self, buf: &mut String) {
        let (offset_x, offset_y) = self.offset;

        // print chars to the screen based on the mask
        for row_idx in 0..self.text_dims.1 {
            write_move_to(buf, offset_x, row_idx + offset_y);

            let mut current_color = self.colors[0];
            write_fg_color(buf, current_color);

            let mut skipping = false;

            for col_idx in 0..self.text_dims.0 {
                let char = self.chars[row_idx][col_idx];
                let will_draw = char != ' ' && self.mask[row_idx][col_idx];

                if will_draw {
                    if skipping {
                        skipping = false;
                        write_move_to(buf, col_idx + offset_x, row_idx + offset_y);
                    }
                    if self.colors[col_idx] != current_color {
                        current_color = self.colors[col_idx];
                        write_fg_color(buf, current_color);
                    }
                    let _ = write!(buf, "{}", self.chars[row_idx][col_idx]);
                } else {
                    skipping = true;
                }
            }

            write_fg_reset(buf);
        }
    }
}

fn get_letter(ch: char) -> Option<&'static [&'static str; LETTER_HEIGHT]> {
    match ch.to_ascii_uppercase() {
        'A' => Some(&A),
        'B' => Some(&B),
        'C' => Some(&C),
        'D' => Some(&D),
        'E' => Some(&E),
        'F' => Some(&F),
        'G' => Some(&G),
        'H' => Some(&H),
        'I' => Some(&I),
        'J' => Some(&J),
        'K' => Some(&K),
        'L' => Some(&L),
        'M' => Some(&M),
        'N' => Some(&N),
        'O' => Some(&O),
        'P' => Some(&P),
        'Q' => Some(&Q),
        'R' => Some(&R),
        'S' => Some(&S),
        'T' => Some(&T),
        'U' => Some(&U),
        'V' => Some(&V),
        'W' => Some(&W),
        'X' => Some(&X),
        'Y' => Some(&Y),
        'Z' => Some(&Z),
        '!' => Some(&EXCLAIM),
        _ => None,
    }
}

// ALPHABET
// Each letter is 6 lines tall. Widths vary by letter.

#[rustfmt::skip]
const A: [&str; 6] = [
    " █████╗ ",
    "██╔══██╗",
    "███████║",
    "██╔══██║",
    "██║  ██║",
    "╚═╝  ╚═╝",
];

#[rustfmt::skip]
const B: [&str; 6] = [
    "██████╗ ",
    "██╔══██╗",
    "██████╔╝",
    "██╔══██╗",
    "██████╔╝",
    "╚═════╝ ",
];

#[rustfmt::skip]
const C: [&str; 6] = [
    " ██████╗",
    "██╔════╝",
    "██║     ",
    "██║     ",
    "╚██████╗",
    " ╚═════╝",
];

#[rustfmt::skip]
const D: [&str; 6] = [
    "██████╗ ",
    "██╔══██╗",
    "██║  ██║",
    "██║  ██║",
    "██████╔╝",
    "╚═════╝ ",
];

#[rustfmt::skip]
const E: [&str; 6] = [
    "███████╗",
    "██╔════╝",
    "█████╗  ",
    "██╔══╝  ",
    "███████╗",
    "╚══════╝",
];

#[rustfmt::skip]
const F: [&str; 6] = [
    "███████╗",
    "██╔════╝",
    "█████╗  ",
    "██╔══╝  ",
    "██║     ",
    "╚═╝     ",
];

#[rustfmt::skip]
const G: [&str; 6] = [
    " ██████╗ ",
    "██╔════╝ ",
    "██║  ███╗",
    "██║   ██║",
    "╚██████╔╝",
    " ╚═════╝ ",
];

#[rustfmt::skip]
const H: [&str; 6] = [
    "██╗  ██╗",
    "██║  ██║",
    "███████║",
    "██╔══██║",
    "██║  ██║",
    "╚═╝  ╚═╝",
];

#[rustfmt::skip]
const I: [&str; 6] = [
    "██╗",
    "██║",
    "██║",
    "██║",
    "██║",
    "╚═╝",
];

#[rustfmt::skip]
const J: [&str; 6] = [
    "     ██╗",
    "     ██║",
    "     ██║",
    "██   ██║",
    "╚█████╔╝",
    " ╚════╝ ",
];

#[rustfmt::skip]
const K: [&str; 6] = [
    "██╗  ██╗",
    "██║ ██╔╝",
    "█████╔╝ ",
    "██╔═██╗ ",
    "██║  ██╗",
    "╚═╝  ╚═╝",
];

#[rustfmt::skip]
const L: [&str; 6] = [
    "██╗     ",
    "██║     ",
    "██║     ",
    "██║     ",
    "███████╗",
    "╚══════╝",
];

#[rustfmt::skip]
const M: [&str; 6] = [
    "███╗   ███╗",
    "████╗ ████║",
    "██╔████╔██║",
    "██║╚██╔╝██║",
    "██║ ╚═╝ ██║",
    "╚═╝     ╚═╝",
];

#[rustfmt::skip]
const N: [&str; 6] = [
    "███╗   ██╗",
    "████╗  ██║",
    "██╔██╗ ██║",
    "██║╚██╗██║",
    "██║ ╚████║",
    "╚═╝  ╚═══╝",
];

#[rustfmt::skip]
const O: [&str; 6] = [
    " ██████╗ ",
    "██╔═══██╗",
    "██║   ██║",
    "██║   ██║",
    "╚██████╔╝",
    " ╚═════╝ ",
];

#[rustfmt::skip]
const P: [&str; 6] = [
    "██████╗ ",
    "██╔══██╗",
    "██████╔╝",
    "██╔═══╝ ",
    "██║     ",
    "╚═╝     ",
];

#[rustfmt::skip]
const Q: [&str; 6] = [
    " ██████╗  ",
    "██╔═══██╗ ",
    "██║   ██║ ",
    "██║▄▄ ██║ ",
    "╚██████╔╝ ",
    " ╚══▀▀═╝  ",
];

#[rustfmt::skip]
const R: [&str; 6] = [
    "██████╗ ",
    "██╔══██╗",
    "██████╔╝",
    "██╔══██╗",
    "██║  ██║",
    "╚═╝  ╚═╝",
];

#[rustfmt::skip]
const S: [&str; 6] = [
    "███████╗",
    "██╔════╝",
    "███████╗",
    "╚════██║",
    "███████║",
    "╚══════╝",
];

#[rustfmt::skip]
const T: [&str; 6] = [
    "████████╗",
    "╚══██╔══╝",
    "   ██║   ",
    "   ██║   ",
    "   ██║   ",
    "   ╚═╝   ",
];

#[rustfmt::skip]
const U: [&str; 6] = [
    "██╗   ██╗",
    "██║   ██║",
    "██║   ██║",
    "██║   ██║",
    "╚██████╔╝",
    " ╚═════╝ ",
];

#[rustfmt::skip]
const V: [&str; 6] = [
    "██╗   ██╗",
    "██║   ██║",
    "██║   ██║",
    "╚██╗ ██╔╝",
    " ╚████╔╝ ",
    "  ╚═══╝  ",
];

#[rustfmt::skip]
const W: [&str; 6] = [
    "██╗    ██╗",
    "██║    ██║",
    "██║ █╗ ██║",
    "██║███╗██║",
    "╚███╔███╔╝",
    " ╚══╝╚══╝ ",
];

#[rustfmt::skip]
const X: [&str; 6] = [
    "██╗  ██╗",
    "╚██╗██╔╝",
    " ╚███╔╝ ",
    " ██╔██╗ ",
    "██╔╝ ██╗",
    "╚═╝  ╚═╝",
];

#[rustfmt::skip]
const Y: [&str; 6] = [
    "██╗   ██╗",
    "╚██╗ ██╔╝",
    " ╚████╔╝ ",
    "  ╚██╔╝  ",
    "   ██║   ",
    "   ╚═╝   ",
];

#[rustfmt::skip]
const Z: [&str; 6] = [
    "███████╗",
    "╚══███╔╝",
    "  ███╔╝ ",
    " ███╔╝  ",
    "███████╗",
    "╚══════╝",
];

#[rustfmt::skip]
const EXCLAIM: [&str; 6] = [
    "██╗",
    "██║",
    "██║",
    "╚═╝",
    "██╗",
    "╚═╝",
];

use crate::party::{PartyEntry, PartyInfo, PartyRenderer};

use super::{Palette, RenderContext, random_pick};

const WORDS: &[&str] = &["NICE!", "SWEET!", "SICK!", "DOPE!", "COOL!", "YEAH!"];

const LETTER_SPACING: usize = 0;
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
    renderer: PartyRenderer::Inline { render },
};

fn render(_ctx: &RenderContext, palette: &Palette) -> bool {
    let word = random_pick(WORDS);
    let offset = palette.random_offset();

    // gather lines by scanning through each letter
    // NOTE: adds a extra space before the word
    let mut lines = vec![String::from(" "); LETTER_HEIGHT];

    for (idx, ch) in word.chars().enumerate() {
        let color = palette.get_color(offset + idx);
        if let Some(letter) = get_letter(ch) {
            for (i, letter_line) in letter.iter().enumerate() {
                let segment = format!("{color}{letter_line}{}", &" ".repeat(LETTER_SPACING));
                lines[i].push_str(&segment);
            }
        }
    }

    // print each line
    for line in lines {
        println!("{}", line);
    }

    true
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

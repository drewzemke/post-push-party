use super::{random_pick, Party, PartyColor, RenderContext, BOLD, RESET};

/// Prints a random word in large ASCII art letters.
pub struct BigText;

const WORDS: &[&str] = &["NICE!", "SWEET!", "SICK!", "DOPE!", "COOL!", "YEAH!"];

const LETTER_SPACING: usize = 0;
const LETTER_HEIGHT: usize = 6;

impl Party for BigText {
    fn id(&self) -> &'static str {
        "big_text"
    }

    fn name(&self) -> &'static str {
        "Big Text"
    }

    fn description(&self) -> &'static str {
        "Shows a random word in big ASCII art letters."
    }

    fn cost(&self) -> u64 {
        2000
    }

    fn supports_color(&self) -> bool {
        true
    }

    fn render(&self, _ctx: &RenderContext, color: &PartyColor) {
        let word = random_pick(WORDS);
        let offset = color.random_offset();

        // gather lines by scanning through each letter
        // NOTE: adds a extra space before the word
        let mut lines = vec![String::from(" "); LETTER_HEIGHT];

        for (idx, ch) in word.chars().enumerate() {
            let color = color.get(offset + idx);
            if let Some(letter) = get_letter(ch) {
                for (i, letter_line) in letter.iter().enumerate() {
                    lines[i].push_str(color);
                    lines[i].push_str(letter_line);

                    if LETTER_SPACING > 0 {
                        lines[i].push_str(&" ".repeat(LETTER_SPACING));
                    }
                }
            }
        }

        // print each line
        print!("{}", BOLD);
        for line in lines {
            println!("{}", line);
        }
        print!("{}", RESET);
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

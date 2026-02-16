use std::fmt::Display;

const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const ITALIC: &str = "\x1b[3m";

#[expect(dead_code)]
const RESET_ALL: &str = "\x1b[0m";
const RESET_FONT: &str = "\x1b[22m";
pub const RESET_COLOR: &str = "\x1b[39m";

#[expect(dead_code)]
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
#[expect(dead_code)]
const BLUE: &str = "\x1b[34m";
const MAGENTA: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";
const GRAY: &str = "\x1b[90m";

#[expect(dead_code)]
const BRIGHT_RED: &str = "\x1b[91m";
#[expect(dead_code)]
const BRIGHT_GREEN: &str = "\x1b[92m";
const BRIGHT_YELLOW: &str = "\x1b[93m";
#[expect(dead_code)]
const BRIGHT_BLUE: &str = "\x1b[94m";
const BRIGHT_MAGENTA: &str = "\x1b[95m";
#[expect(dead_code)]
const BRIGHT_CYAN: &str = "\x1b[96m";
#[expect(dead_code)]
const BRIGHT_WHITE: &str = "\x1b[97m";

pub fn bold(s: impl Display) -> String {
    format!("{BOLD}{s}{RESET_FONT}")
}

pub fn italic(s: impl Display) -> String {
    format!("{ITALIC}{s}{RESET_FONT}")
}

pub fn dim(s: impl Display) -> String {
    format!("{DIM}{s}{RESET_FONT}")
}

pub fn white(s: impl Display) -> String {
    format!("{RESET_COLOR}{s}")
}

pub fn green(s: impl Display) -> String {
    format!("{GREEN}{s}{RESET_COLOR}")
}

pub fn magenta(s: impl Display) -> String {
    format!("{MAGENTA}{s}{RESET_COLOR}")
}

pub fn cyan(s: impl Display) -> String {
    format!("{CYAN}{s}{RESET_COLOR}")
}

pub fn yellow(s: impl Display) -> String {
    format!("{YELLOW}{s}{RESET_COLOR}")
}

pub fn gray(s: impl Display) -> String {
    format!("{GRAY}{s}{RESET_COLOR}")
}

pub fn bright_magenta(s: impl Display) -> String {
    format!("{BRIGHT_MAGENTA}{s}{RESET_COLOR}")
}

pub fn bright_yellow(s: impl Display) -> String {
    format!("{BRIGHT_YELLOW}{s}{RESET_COLOR}")
}

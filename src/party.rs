use crate::state::State;

const EXCLAMATIONS: &[&str] = &[
    "NICE!", "AWESOME!", "GREAT JOB!", "FANTASTIC!", "WOOHOO!",
    "AMAZING!", "BRILLIANT!", "SUPERB!", "EXCELLENT!", "WONDERFUL!",
];

const QUOTES: &[&str] = &[
    "The only way to do great work is to love what you do.",
    "First, solve the problem. Then, write the code.",
    "Code is like humor. When you have to explain it, it's bad.",
    "Simplicity is the soul of efficiency.",
    "Make it work, make it right, make it fast.",
    "Any fool can write code that a computer can understand.",
    "The best error message is the one that never shows up.",
    "Delete code bravely.",
    "Weeks of coding can save you hours of planning.",
    "It works on my machine!",
];

const COLORS: &[&str] = &[
    "\x1b[31m", // red
    "\x1b[32m", // green
    "\x1b[33m", // yellow
    "\x1b[34m", // blue
    "\x1b[35m", // magenta
    "\x1b[36m", // cyan
];

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";

fn random_pick<T>(items: &[T]) -> &T {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as usize)
        .unwrap_or(0);
    &items[seed % items.len()]
}

pub fn display(state: &State, commits: u64, points_earned: u64) {
    match state.party_level {
        0 => display_basic(commits, points_earned),
        1 => display_colorful(commits, points_earned),
        2 => display_quotes(commits, points_earned),
        _ => display_big_text(commits, points_earned),
    }
}

fn display_basic(commits: u64, points_earned: u64) {
    println!();
    println!("🎉 You earned {} party points!", points_earned);
    if commits > 1 {
        println!("   ({} commits)", commits);
    }
    println!();
    println!("Run `party` to see your total!");
    println!();
}

fn display_colorful(commits: u64, points_earned: u64) {
    let color = random_pick(COLORS);
    let exclaim = random_pick(EXCLAMATIONS);

    println!();
    println!("{}{}{} {}", BOLD, color, exclaim, RESET);
    println!();
    println!("{}🎉 You earned {} party points!{}", color, points_earned, RESET);
    if commits > 1 {
        println!("   ({} commits)", commits);
    }
    println!();
}

fn display_quotes(commits: u64, points_earned: u64) {
    let color = random_pick(COLORS);
    let exclaim = random_pick(EXCLAMATIONS);
    let quote = random_pick(QUOTES);

    println!();
    println!("{}{}{} {}", BOLD, color, exclaim, RESET);
    println!();
    println!("{}🎉 You earned {} party points!{}", color, points_earned, RESET);
    if commits > 1 {
        println!("   ({} commits)", commits);
    }
    println!();
    println!("{}\"{}\"{}",  "\x1b[3m", quote, RESET); // italic
    println!();
}

fn display_big_text(commits: u64, points_earned: u64) {
    let color = random_pick(COLORS);
    let quote = random_pick(QUOTES);

    println!();
    println!("{}{}", color, BOLD);
    println!(" ███╗   ██╗██╗ ██████╗███████╗██╗");
    println!(" ████╗  ██║██║██╔════╝██╔════╝██║");
    println!(" ██╔██╗ ██║██║██║     █████╗  ██║");
    println!(" ██║╚██╗██║██║██║     ██╔══╝  ╚═╝");
    println!(" ██║ ╚████║██║╚██████╗███████╗██╗");
    println!(" ╚═╝  ╚═══╝╚═╝ ╚═════╝╚══════╝╚═╝");
    println!("{}", RESET);
    println!();
    println!("🎉 +{} party points!", points_earned);
    if commits > 1 {
        println!("   ({} commits)", commits);
    }
    println!();
    println!("{}\"{}\"{}",  "\x1b[3m", quote, RESET);
    println!();
}

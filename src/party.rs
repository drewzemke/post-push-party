use crate::state::{PartyFeature, State};

const EXCLAMATIONS: &[&str] = &[
    "NICE!",
    "AWESOME!",
    "GREAT JOB!",
    "FANTASTIC!",
    "WOOHOO!",
    "AMAZING!",
    "BRILLIANT!",
    "SUPERB!",
    "EXCELLENT!",
    "WONDERFUL!",
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
    use rand::prelude::IndexedRandom;
    items.choose(&mut rand::rng()).unwrap()
}

pub fn display(state: &State, commits_pushed: u64, commits_counted: u64, points_earned: u64) {
    let use_exclamations = state.is_enabled(PartyFeature::Exclamations);
    let use_quotes = state.is_enabled(PartyFeature::Quotes);
    let use_big_text = state.is_enabled(PartyFeature::BigText);

    let use_color = use_exclamations || use_big_text; // color when exclamations or big text
    let color = if use_color { random_pick(COLORS) } else { "" };
    let reset = if use_color { RESET } else { "" };
    let bold = if use_color { BOLD } else { "" };

    println!();

    // big text or exclamation
    if use_big_text {
        println!("{}{}", color, bold);
        println!(" ███╗   ██╗██╗ ██████╗███████╗██╗");
        println!(" ████╗  ██║██║██╔════╝██╔════╝██║");
        println!(" ██╔██╗ ██║██║██║     █████╗  ██║");
        println!(" ██║╚██╗██║██║██║     ██╔══╝  ╚═╝");
        println!(" ██║ ╚████║██║╚██████╗███████╗██╗");
        println!(" ╚═╝  ╚═══╝╚═╝ ╚═════╝╚══════╝╚═╝");
        println!("{}", reset);
        println!();
    } else if use_exclamations {
        let exclaim = random_pick(EXCLAMATIONS);
        println!("{}{}{} {}", bold, color, exclaim, reset);
        println!();
    }

    // summary (always shown)
    if use_big_text {
        println!("🎉 +{} party points!", points_earned);
    } else {
        println!(
            "{}🎉 You earned {} party points!{}",
            color, points_earned, reset
        );
    }
    if commits_pushed != commits_counted {
        println!(
            "   ({} commits pushed, {} counted)",
            commits_pushed, commits_counted
        );
    } else if commits_counted > 1 {
        println!("   ({} commits)", commits_counted);
    }
    println!();

    // quote
    if use_quotes {
        let quote = random_pick(QUOTES);
        println!("\x1b[3m\"{}\"\x1b[0m", quote);
        println!();
    }

    // call to action (only if no fancy output)
    if !use_exclamations && !use_quotes && !use_big_text {
        println!("Run `party` to see your total!");
        println!();
    }
}

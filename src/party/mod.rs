mod base;
mod big_text;
mod breakdown;
mod color;
mod context;
mod exclamation;
mod quotes;
mod stats;

pub use color::Color as PartyColor;
pub use context::RenderContext;

use base::Base;
use big_text::BigText;
use breakdown::Breakdown;
use exclamation::Exclamation;
use quotes::Quotes;
use stats::Stats;

use crate::party::color::ALL_COLORS;

pub trait Party: Sync {
    /// unique identifier for state storage
    fn id(&self) -> &'static str;

    /// display name for the UI
    fn name(&self) -> &'static str;

    /// description for the UI
    fn description(&self) -> &'static str;

    /// unlock cost
    fn cost(&self) -> u64;

    /// whether or not the color of the output of this party is configurable
    #[expect(dead_code)]
    fn supports_color(&self) -> bool;

    /// prints the output of this party to stdout
    fn render(&self, ctx: &RenderContext, color: &PartyColor);
}

// static instances
static BASE: Base = Base;
static BIG_TEXT: BigText = BigText;
static BREAKDOWN: Breakdown = Breakdown;
static EXCLAMATION: Exclamation = Exclamation;
static QUOTES: Quotes = Quotes;
static STATS: Stats = Stats;

// all parties in order
pub static ALL_PARTIES: &[&'static dyn Party] =
    &[&BASE, &BREAKDOWN, &STATS, &EXCLAMATION, &BIG_TEXT, &QUOTES];

// display utilities
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const ITALICS: &str = "\x1b[3m";
const CYAN: &str = "\x1b[36m";
const MAGENTA: &str = "\x1b[35m";
const YELLOW: &str = "\x1b[33m";
const GRAY: &str = "\x1b[90m";

fn random_pick<T>(items: &[T]) -> &T {
    use rand::prelude::IndexedRandom;
    items.choose(&mut rand::rng()).unwrap()
}

/// renders every enabled party
pub fn display(ctx: &RenderContext) {
    let enabled_parties = ALL_PARTIES
        .iter()
        .filter(|party| ctx.state.is_party_enabled(party.id()))
        .map(|p| *p);

    // print a blank line before starting any party
    println!();

    for party in enabled_parties {
        // HACK: choosing a random color for now from a predefined list;
        // will implement color unlocking / selection later
        let color = random_pick(ALL_COLORS);

        party.render(ctx, color);

        // print a blank line between parties (and after the last one)
        println!();
    }
}

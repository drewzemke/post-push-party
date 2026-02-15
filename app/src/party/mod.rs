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

use crate::{party::color::ALL_COLORS, state::ColorSelection};

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

/// chooses a random element of a NONEMPTY list
fn random_pick<T>(items: &[T]) -> &T {
    use rand::prelude::IndexedRandom;
    items
        .choose(&mut rand::rng())
        .expect("list must be nonempty")
}

/// renders every enabled party
pub fn display(ctx: &RenderContext) {
    let enabled_parties = ALL_PARTIES
        .iter()
        .filter(|party| ctx.state.is_party_enabled(party.id()))
        .copied();

    // print a blank line before starting any party
    println!();

    for party in enabled_parties {
        // resolve a color for this party based on the user's configuration
        let color_selection = ctx.state.selected_color(party.id());

        let color_name = match color_selection {
            // if the user wants a random color, pick one from the list of available colors for this party
            Some(ColorSelection::Random) => {
                let unlocked_colors = ctx
                    .state
                    .unlocked_colors(party.id())
                    .map(|set| set.iter().collect::<Vec<_>>())
                    .unwrap_or_default();

                if unlocked_colors.is_empty() {
                    PartyColor::WHITE.name().to_string()
                } else {
                    random_pick(&unlocked_colors).to_string()
                }
            }

            Some(ColorSelection::Specific(color_name)) => color_name.to_string(),

            // if nothing has been chosen, go with white
            None => PartyColor::WHITE.name().to_string(),
        };

        // look it up the resolved color name in the list of colors,
        // falling back to white if not found (which shouldn't happen... right?)
        let color = ALL_COLORS
            .iter()
            .find(|&&c| c.name() == color_name)
            .unwrap_or(&&PartyColor::WHITE);

        party.render(ctx, color);

        // print a blank line between parties (and after the last one)
        println!();
    }
}

mod base;
mod big_text;
mod breakdown;
mod context;
mod exclamation;
mod fireworks;
pub(crate) mod palette;
mod quotes;
pub(crate) mod stats;
mod style;

pub use context::RenderContext;
pub use palette::Palette;

use base::Base;
use big_text::BigText;
use breakdown::Breakdown;
use exclamation::Exclamation;
use fireworks::Fireworks;
use quotes::Quotes;
use stats::Stats;

use crate::{party::palette::ALL_PALETTES, state::PaletteSelection};

pub trait Party: Sync {
    /// unique identifier for state storage
    fn id(&self) -> &'static str;

    /// display name for the UI
    fn name(&self) -> &'static str;

    /// description for the UI
    fn description(&self) -> &'static str;

    /// unlock cost
    fn cost(&self) -> u64;

    /// whether or not the color palette of the output of this party is configurable
    #[expect(dead_code)]
    fn supports_color(&self) -> bool;

    /// prints the output of this party to stdout
    /// returns whether or not permanent content was printed to the terminal
    fn render(&self, ctx: &RenderContext, palette: &Palette) -> bool;
}

// static instances
pub static BASE: Base = Base;
static BIG_TEXT: BigText = BigText;
static BREAKDOWN: Breakdown = Breakdown;
static EXCLAMATION: Exclamation = Exclamation;
static QUOTES: Quotes = Quotes;
static STATS: Stats = Stats;
pub static FIREWORKS: Fireworks = Fireworks;

// all parties in order
pub static ALL_PARTIES: &[&'static dyn Party] = &[
    &BASE,
    &BREAKDOWN,
    &STATS,
    &EXCLAMATION,
    &BIG_TEXT,
    &QUOTES,
    &FIREWORKS,
];

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
        let palette_selection = ctx.state.selected_palette(party.id());

        let palette_name = match palette_selection {
            // if the user wants a random palette, pick one from the list of available palettes for this party
            Some(PaletteSelection::Random) => {
                let unlocked_palettes = ctx
                    .state
                    .unlocked_palettes(party.id())
                    .map(|set| set.iter().collect::<Vec<_>>())
                    .unwrap_or_default();

                if unlocked_palettes.is_empty() {
                    Palette::WHITE.name().to_string()
                } else {
                    random_pick(&unlocked_palettes).to_string()
                }
            }

            Some(PaletteSelection::Specific(color_name)) => color_name.to_string(),

            // if nothing has been chosen, go with white
            None => Palette::WHITE.name().to_string(),
        };

        // look it up the resolved palette name in the list of palettes,
        // falling back to white if not found (which shouldn't happen... right?)
        let palette = ALL_PALETTES
            .iter()
            .find(|&&c| c.name() == palette_name)
            .unwrap_or(&&Palette::WHITE);

        let printed_text = party.render(ctx, palette);

        // print blank line after each party that prints text
        if printed_text {
            println!();
        }
    }
}

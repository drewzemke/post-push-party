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
    fn supports_color(&self) -> bool;

    /// prints the output of this party to stdout
    /// returns whether or not permanent content was printed to the terminal
    fn render(&self, ctx: &RenderContext, palette: &Palette) -> bool;
}

/// Metadata about a party
pub struct PartyInfo {
    /// unlock cost
    pub cost: u64,

    /// description for the UI
    pub description: &'static str,

    /// unique identifier for state storage
    pub id: &'static str,

    /// display name for the UI
    pub name: &'static str,

    /// whether or not the color palette of the output of this party is configurable
    pub supports_color: bool,
}

pub trait FullscreenPartyRenderer {
    /// used to determine the order in which a party is drawn to the screen
    /// relative to others (smaller is earlier in draw order)
    fn z_index(&self) -> u32;

    /// updates a party's internal state (for animating)
    /// return false to indicate that the animation is "done"
    fn update(&mut self, dt: std::time::Duration) -> bool;

    /// renders the party to the screen based on its current animation
    /// state. parties should not clear the screen and should only render
    /// to cells that have content, leaving blank space otherwise
    fn render(&self, buf: &mut String);
}

pub enum PartyRenderer {
    Inline {
        /// prints the output of this party to stdout
        /// returns whether or not permanent content was printed to the terminal
        // FIXME: we don't need the return value anymore
        render: fn(&RenderContext<'_>, &Palette) -> bool,
    },
    Fullscreen {
        /// factory function that's called per party run
        ///
        /// params are (width, height, palette)
        create: fn(u16, u16, &Palette) -> Box<dyn FullscreenPartyRenderer>,
    },
}

// TODO: rename to just `Party`
pub struct PartyEntry {
    pub info: PartyInfo,
    pub renderer: PartyRenderer,
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
pub static ALL_PARTIES: &[&PartyEntry] = &[
    &base::BASE_PARTY,
    &breakdown::BREAKDOWN_PARTY,
    &stats::STATS_PARTY,
    &exclamation::EXCLAMATION_PARTY,
    &big_text::BIG_TEXT_PARTY,
    &quotes::QUOTES_PARTY,
    // &FIREWORKS,
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
        .filter(|party| ctx.state.is_party_enabled(party.info.id))
        .copied();

    render_inline_parties(ctx, enabled_parties);
}

fn render_inline_parties(
    ctx: &RenderContext<'_>,
    enabled_parties: impl Iterator<Item = &'static PartyEntry>,
) {
    // print a blank line before printing any party
    println!();

    // render all of the inline parties
    for party in enabled_parties {
        let PartyRenderer::Inline { render } = party.renderer else {
            continue;
        };

        let palette = get_palette(party, ctx);
        let printed_text = render(ctx, palette);

        // print blank line after each party that prints text
        if printed_text {
            println!();
        }
    }
}

/// resolves a color for this party based on the user's configuration
fn get_palette(party: &PartyEntry, ctx: &RenderContext<'_>) -> &'static Palette {
    let palette_selection = ctx.state.selected_palette(party.info.id);

    let palette_id = match palette_selection {
        // if the user wants a random palette, pick one from the list of available palettes for this party
        Some(PaletteSelection::Random) => {
            let unlocked_palettes = ctx
                .state
                .unlocked_palettes(party.info.id)
                .map(|set| set.iter().collect::<Vec<_>>())
                .unwrap_or_default();

            if unlocked_palettes.is_empty() {
                Palette::WHITE_ANSI.id().to_string()
            } else {
                random_pick(&unlocked_palettes).to_string()
            }
        }

        Some(PaletteSelection::Specific(color_name)) => color_name.to_string(),

        // if nothing has been chosen, go with white
        None => Palette::WHITE_ANSI.id().to_string(),
    };

    // look it up the resolved palette name in the list of palettes,
    // falling back to white if not found (which shouldn't happen... right?)
    ALL_PALETTES
        .iter()
        .find(|&&c| c.id() == palette_id)
        .unwrap_or(&&Palette::WHITE_ANSI)
}

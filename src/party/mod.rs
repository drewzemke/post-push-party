use crate::scoring::{AppliedBonus, PointsBreakdown};

mod base;
mod color;
mod context;
mod exclamation;

pub use color::Color as PartyColor;
pub use context::RenderContext;

use base::Base;
use exclamation::Exclamation;

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
    fn supports_color(&self) -> bool;

    /// prints the output of this party to stdout
    fn render(&self, ctx: &RenderContext, color: PartyColor);
}

// static instances
static BASE: Base = Base;
static EXCLAMATION: Exclamation = Exclamation;

// all parties in order
pub static ALL_PARTIES: &[&'static dyn Party] = &[&BASE, &EXCLAMATION];

// display utilities
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";

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

    for party in enabled_parties {
        // TODO: choose color
        let color = PartyColor::White;
        party.render(ctx, color);
    }

    // // big text or exclamation
    // if use_big_text {
    //     println!("{}{}", color, bold);
    //     println!(" ███╗   ██╗██╗ ██████╗███████╗██╗");
    //     println!(" ████╗  ██║██║██╔════╝██╔════╝██║");
    //     println!(" ██╔██╗ ██║██║██║     █████╗  ██║");
    //     println!(" ██║╚██╗██║██║██║     ██╔══╝  ╚═╝");
    //     println!(" ██║ ╚████║██║╚██████╗███████╗██╗");
    //     println!(" ╚═╝  ╚═══╝╚═╝ ╚═════╝╚══════╝╚═╝");
    //     println!("{}", reset);
    //     println!();
    // } else if use_exclamations {
    //     let exclaim = random_pick(EXCLAMATIONS);
    //     println!("{}{}{} {}", bold, color, exclaim, reset);
    //     println!();
    // }

    // // main points line
    // if breakdown.total > 0 {
    //     println!(
    //         "{}🎉 You earned {} party points!{}",
    //         color, breakdown.total, reset
    //     );
    //     println!();

    //     // breakdown: commits × points per commit
    //     let commit_word = if breakdown.commits == 1 {
    //         "commit"
    //     } else {
    //         "commits"
    //     };
    //     let point_word = if breakdown.points_per_commit == 1 {
    //         "point"
    //     } else {
    //         "points"
    //     };
    //     println!(
    //         "   {} {} × {} {} per commit",
    //         breakdown.commits, commit_word, breakdown.points_per_commit, point_word
    //     );

    //     // flat bonuses first (they add to base)
    //     for bonus in &breakdown.applied {
    //         if let AppliedBonus::FlatBonus {
    //             name,
    //             points,
    //             count,
    //         } = bonus
    //         {
    //             println!("   + {} {} ({} ×)", points, name, count);
    //         }
    //     }

    //     // multiplier bonuses (they multiply the total)
    //     for bonus in &breakdown.applied {
    //         if let AppliedBonus::Multiplier { name, value } = bonus {
    //             println!("   × {} {}", value, name);
    //         }
    //     }
    //     println!();
    // } else {
    //     println!("{}🎉 Pushed! (already counted){}", color, reset);
    //     println!();
    // }

    // // quote
    // if use_quotes {
    //     let quote = random_pick(QUOTES);
    //     println!("\x1b[3m\"{}\"\x1b[0m", quote);
    //     println!();
    // }

    // // call to action (only if no fancy output)
    // if !use_exclamations && !use_quotes && !use_big_text {
    //     println!("Run `party` to see your total!");
    //     println!();
    // }
}

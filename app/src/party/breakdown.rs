use crate::scoring::AppliedBonus;

use super::{Party, PartyColor, RenderContext};

/// shows how the total points were calculated for a push, including bonuses
pub struct Breakdown;

impl Party for Breakdown {
    fn id(&self) -> &'static str {
        "breakdown"
    }

    fn name(&self) -> &'static str {
        "Points Breakdown"
    }

    fn description(&self) -> &'static str {
        "Shows which bonuses applied to your push and how your total was calculated."
    }

    fn cost(&self) -> u64 {
        50
    }

    fn supports_color(&self) -> bool {
        false
    }

    fn render(&self, ctx: &RenderContext, _color: &PartyColor) -> bool {
        let breakdown = ctx.breakdown;
        let commits = breakdown.commits;
        let points_per_commit = breakdown.points_per_commit;

        // TODO: colorize

        //  N commits × M points per commit
        let commit_word = if commits == 1 { "commit" } else { "commits" };
        let point_word = if points_per_commit == 1 {
            "point"
        } else {
            "points"
        };
        println!("   {commits} {commit_word} × {points_per_commit} {point_word} per commit",);

        // flat bonuses first (they add to base)
        for bonus in &breakdown.applied {
            if let AppliedBonus::FlatBonus {
                name,
                points,
                count,
            } = bonus
            {
                println!("   + {points} {name} ({count} ×)");
            }
        }

        // multiplier bonuses (multiply the total)
        for bonus in &breakdown.applied {
            if let AppliedBonus::Multiplier { name, value } = bonus {
                println!("   × {value} {name}");
            }
        }

        // TODO:
        // add "= <total>"?

        true
    }
}

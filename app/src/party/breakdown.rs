use crate::scoring::AppliedBonus;

use super::{
    BOLD, BRIGHT_MAGENTA, BRIGHT_YELLOW, DIM, GREEN, MAGENTA, NORMAL, Party, PartyColor, RESET,
    RenderContext, YELLOW,
};

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

        //  N commits × M points per commit
        let commit_word = if commits == 1 { "commit" } else { "commits" };
        let point_word = if points_per_commit == 1 {
            "point"
        } else {
            "points"
        };
        println!(
            "  {BOLD}{GREEN}{commits}{RESET} {commit_word} {MAGENTA}×{RESET} {BOLD}{BRIGHT_MAGENTA}{points_per_commit}{RESET} {point_word} per commit",
        );

        // flat bonuses first (they add to base)
        for bonus in &breakdown.applied {
            if let AppliedBonus::FlatBonus {
                name,
                points,
                count,
            } = bonus
            {
                let extra_words = if *count > 1 {
                    format!("{DIM} (applied {count} times){RESET}")
                } else {
                    String::new()
                };
                println!(
                    "   {DIM}{MAGENTA}+ {BOLD}{BRIGHT_MAGENTA}{points}{RESET} {name}{extra_words}"
                );
            }
        }

        // multiplier bonuses (multiply the total)
        for bonus in &breakdown.applied {
            if let AppliedBonus::Multiplier { name, value } = bonus {
                println!("   {DIM}{MAGENTA}× {BOLD}{BRIGHT_MAGENTA}{value}{RESET} {name}");
            }
        }

        println!(
            "   {DIM}={NORMAL} {BOLD}{BRIGHT_YELLOW}{}{NORMAL} {YELLOW}P{RESET}",
            breakdown.total
        );

        true
    }
}

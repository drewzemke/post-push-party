use crate::scoring::AppliedBonus;

use super::{
    Party, PartyColor, RenderContext,
    style::{bold, bright_magenta, bright_yellow, dim, green, magenta, yellow},
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

        let plus = dim(magenta("+"));
        let times = dim(magenta("×"));

        //  N commits × M points per commit
        let commit_word = if commits == 1 { "commit" } else { "commits" };
        let point_word = if points_per_commit == 1 {
            "point"
        } else {
            "points"
        };

        let commits = bold(green(commits));
        let points_per_commit = bold(bright_magenta(points_per_commit));
        println!("  {commits} {commit_word} {times} {points_per_commit} {point_word} per commit",);

        // flat bonuses first (they add to base)
        for bonus in &breakdown.applied {
            if let AppliedBonus::FlatBonus {
                name,
                points,
                count,
            } = bonus
            {
                let extra_words = if *count > 1 {
                    dim(format!(" (applied {count} times)"))
                } else {
                    String::new()
                };
                let points = bold(bright_magenta(points));
                println!("   {plus} {points} {name}{extra_words}");
            }
        }

        // multiplier bonuses (multiply the total)
        for bonus in &breakdown.applied {
            if let AppliedBonus::Multiplier { name, value } = bonus {
                let value = bold(bright_magenta(value));
                println!("   {times} {value} {name}");
            }
        }

        // total
        let equals = dim("=");
        let total = bold(bright_yellow(breakdown.total));
        let p = yellow("P");
        println!("   {equals} {total} {p}");

        true
    }
}

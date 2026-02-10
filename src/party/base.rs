use super::{Party, PartyColor, RenderContext};

/// the most basic party that shows how many points were earned
pub struct Base;

impl Party for Base {
    fn id(&self) -> &'static str {
        "base"
    }

    fn name(&self) -> &'static str {
        "Basic Party"
    }

    fn description(&self) -> &'static str {
        "Just shows how many points you earned."
    }

    fn cost(&self) -> u64 {
        // free, unlocked by default
        0
    }

    fn supports_color(&self) -> bool {
        true
    }

    // TODO: use color
    fn render(&self, ctx: &RenderContext, _color: PartyColor) {
        let total = ctx.breakdown.total;
        if total > 0 {
            println!("🎉 You earned {total} party points!");
        } else {
            println!("🎉 Pushed! (already counted)");
        }
    }
}

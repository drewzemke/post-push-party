use super::{
    Party, PartyColor, RenderContext,
    style::{RESET_COLOR, bold},
};

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

    fn render(&self, ctx: &RenderContext, color: &PartyColor) -> bool {
        let offset = color.random_offset();
        let color0 = color.get(offset);
        let color1 = color.get(offset + 1);

        let total = ctx.breakdown.total;
        if total > 0 {
            let points = bold(format!("{color1}{total} party points!{RESET_COLOR}"));
            println!("🎉 {color0}You earned {points}");
        } else {
            println!("🎉 {color0}Pushed! {color1}(already counted){RESET_COLOR}");
        }

        true
    }
}

use super::{
    Palette, Party, RenderContext,
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

    fn render(&self, ctx: &RenderContext, palette: &Palette) -> bool {
        let offset = palette.random_offset();
        let color0 = palette.get_ansi_escape(offset);
        let color1 = palette.get_ansi_escape(offset + 1);

        let total = ctx.breakdown.total;
        if total > 0 {
            let points_word = if total == 1 { "point" } else { "points" };
            let points = bold(format!("{color1}{total} party {points_word}"));
            println!("🎉 {color0}You earned {points}{color0}!{RESET_COLOR}");
        } else {
            println!("🎉 {color0}Pushed! {color1}(already counted){RESET_COLOR}");
        }

        // print some extra text if this push caused the player to earn packs
        if !ctx.pack_thresholds.is_empty() {
            let color2 = palette.get_ansi_escape(offset + 2);
            println!();

            for threshold in &ctx.pack_thresholds {
                let a_pack = bold(format!("{color1}a pack{color0}"));
                let lifetime_points = bold(format!("{color2}{threshold} lifetime points{color0}"));
                println!("   {color0}You earned {a_pack} for reaching {lifetime_points}!");
            }
        }

        true
    }
}

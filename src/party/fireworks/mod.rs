use super::{Palette, Party, RenderContext};

mod runner;
mod sim;

/// a full-screen fireworks display
pub struct Fireworks;

impl Party for Fireworks {
    fn id(&self) -> &'static str {
        "fireworks"
    }

    fn name(&self) -> &'static str {
        "Fireworks Party"
    }

    fn description(&self) -> &'static str {
        "A full-screen fireworks display."
    }

    fn cost(&self) -> u64 {
        10_000
    }

    fn supports_color(&self) -> bool {
        true
    }

    fn render(&self, _ctx: &RenderContext, palette: &Palette) -> bool {
        let _ = runner::run(palette.colors());
        false
    }
}

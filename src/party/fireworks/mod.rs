use super::{Palette, Party, RenderContext};

mod renderer;
mod runner;
mod sim;

/// a full-screen fireworks display
pub struct Fireworks;

impl Party for Fireworks {
    fn id(&self) -> &'static str {
        "fireworks"
    }

    fn name(&self) -> &'static str {
        "Fireworks"
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

    // TODO: use color
    fn render(&self, _ctx: &RenderContext, palette: &Palette) -> bool {
        let colors = palette.colors();
        let _ = runner::run(colors);
        false
    }
}

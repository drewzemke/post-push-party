use tixel::BrailleCanvas;

use crate::party::{
    FullscreenPartyRenderer, PartyEntry, PartyInfo, PartyRenderer, fireworks::sim::FireworksSim,
};

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

pub static FIREWORKS_PARTY: PartyEntry = PartyEntry {
    info: PartyInfo {
        id: "fireworks",
        name: "Fireworks Party",
        description: "A full-screen fireworks display.",
        cost: 10_000,
        supports_color: true,
    },
    renderer: PartyRenderer::Fullscreen {
        create: FireworksRenderer::create,
    },
};

struct FireworksRenderer {
    canvas: BrailleCanvas,
    sim: FireworksSim,
    palette: &'static Palette,
}

impl FireworksRenderer {
    pub fn create(
        width: u16,
        height: u16,
        palette: &'static Palette,
    ) -> Box<dyn FullscreenPartyRenderer> {
        let canvas = BrailleCanvas::new((height as usize, width as usize), (0, 0));
        let sim = FireworksSim::new(canvas.width() as f64, canvas.height() as f64);

        Box::new(Self {
            sim,
            canvas,
            palette,
        })
    }
}

impl FullscreenPartyRenderer for FireworksRenderer {
    fn z_index(&self) -> u32 {
        1
    }

    fn update(&mut self, dt: std::time::Duration) -> bool {
        self.sim.update(dt.as_secs_f64())
    }

    fn render(&mut self, buf: &mut String) {
        // render to canvas
        for p in self.sim.particles() {
            self.canvas.set_f(
                p.x,
                self.canvas.height() as f64 - p.y,
                self.palette.get_color(p.color_idx),
            );
        }

        // render to screen
        // FIXME: need to render without replacing entire string
        *buf = self.canvas.render();
    }
}

use super::{random_pick, Party, PartyColor, RenderContext, BOLD, RESET};

const EXCLAMATIONS: &[&str] = &[
    "AWESOME!",
    "WELL DONE!",
    "FANTASTIC!",
    "WOOHOO!",
    "AMAZING!",
    "BRILLIANT!",
    "SUPERB!",
    "EXCELLENT!",
    "WONDERFUL!",
];

/// the most basic party that shows how many points were earned
pub struct Exclamation;

impl Party for Exclamation {
    fn id(&self) -> &'static str {
        "exclamation"
    }

    fn name(&self) -> &'static str {
        "Exclamation"
    }

    fn description(&self) -> &'static str {
        "Prints an all-caps exclamation to celebrate the push."
    }

    fn cost(&self) -> u64 {
        500
    }

    fn supports_color(&self) -> bool {
        true
    }

    // TODO: use color
    fn render(&self, _ctx: &RenderContext, _color: PartyColor) {
        let exclaim = random_pick(EXCLAMATIONS);
        println!("{}{}{}", BOLD, exclaim, RESET);
    }
}

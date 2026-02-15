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

/// prints an emphatic message in all caps
pub struct Exclamation;

impl Party for Exclamation {
    fn id(&self) -> &'static str {
        "exclamation"
    }

    fn name(&self) -> &'static str {
        "Exclamation"
    }

    fn description(&self) -> &'static str {
        "Prints an emphatic all-caps message to celebrate the push."
    }

    fn cost(&self) -> u64 {
        200
    }

    fn supports_color(&self) -> bool {
        true
    }

    fn render(&self, _ctx: &RenderContext, color: &PartyColor) {
        let offset = color.random_offset();
        let exclaim = random_pick(EXCLAMATIONS);

        // NOTE: adds a extra space before the word
        print!("{BOLD} ");

        for (idx, c) in exclaim.chars().enumerate() {
            let color = color.get(offset + idx);
            print!("{color}{c}")
        }

        println!("{RESET}");
    }
}

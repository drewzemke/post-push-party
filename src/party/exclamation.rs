use super::{
    Palette, Party, RenderContext, random_pick,
    style::{RESET_COLOR, bold},
};

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

    fn render(&self, _ctx: &RenderContext, palette: &Palette) -> bool {
        let offset = palette.random_offset();
        let exclaim = random_pick(EXCLAMATIONS);

        // NOTE: adds a extra space before the word
        let mut str = String::from(" ");

        for (idx, c) in exclaim.chars().enumerate() {
            let color = palette.get_ansi_escape(offset + idx);
            str.push_str(&format!("{color}{c}"));
        }

        println!("{}{RESET_COLOR}", bold(str));

        true
    }
}

use crate::party::{PartyEntry, PartyInfo, PartyRenderer};

use super::{
    Palette, RenderContext, random_pick,
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
pub static EXCLAMATION_PARTY: PartyEntry = PartyEntry {
    info: PartyInfo {
        id: "exclamation",
        name: "Exclamation Party",
        description: "Prints an emphatic all-caps message to celebrate the push.",
        cost: 200,
        supports_color: true,
    },
    renderer: PartyRenderer::Inline { render },
};

fn render(_ctx: &RenderContext, palette: &Palette) -> bool {
    let offset = palette.random_offset();
    let exclaim = random_pick(EXCLAMATIONS);

    // NOTE: adds a extra space before the word
    let mut str = String::from(" ");

    for (idx, c) in exclaim.chars().enumerate() {
        let color = palette.get_color(offset + idx);
        color.write_fg(&mut str);
        str.push(c);
    }

    println!("{}{RESET_COLOR}", bold(str));

    true
}

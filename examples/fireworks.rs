use tixel::Color;

fn main() -> anyhow::Result<()> {
    let colors: [Color; 4] = [
        Color::Rgb(240, 80, 240),
        Color::Rgb(80, 240, 240),
        Color::Rgb(240, 240, 240),
        Color::Rgb(240, 240, 80),
    ];

    post_push_party::party::fireworks::run(&colors)
}

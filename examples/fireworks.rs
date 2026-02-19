fn main() -> anyhow::Result<()> {
    let colors: [String; 4] = [
        "\x1b[38;2;240;80;240m".to_string(),
        "\x1b[38;2;80;240;240m".to_string(),
        "\x1b[38;2;240;240;240m".to_string(),
        "\x1b[38;2;240;240;80m".to_string(),
    ];

    post_push_party::party::fireworks::run(&colors)
}

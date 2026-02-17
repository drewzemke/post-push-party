const COLORS: [&str; 4] = [
    "\x1b[38;2;240;80;240m",
    "\x1b[38;2;80;240;240m",
    "\x1b[38;2;240;240;240m",
    "\x1b[38;2;240;240;80m",
];

fn main() -> anyhow::Result<()> {
    post_push_party::party::fireworks::run(&COLORS)
}

use rand::RngExt;

enum Color {
    Rgb(u8, u8, u8),
    Ansi(u8),
}

macro_rules! rgb {
    ($r: literal, $g: literal, $b: literal) => {
        Color::Rgb($r, $g, $b)
    };
}

macro_rules! ansi {
    ($code: literal) => {
        Color::Ansi($code)
    };
}

impl Color {
    fn to_ansi_escape(&self) -> String {
        match self {
            Color::Rgb(r, g, b) => format!("\x1b[38;2;{r};{g};{b}m"),
            Color::Ansi(v) => format!("\x1b[{v}m"),
        }
    }
}

pub struct Palette {
    name: &'static str,
    colors: &'static [Color],
}

impl Palette {
    const fn new(name: &'static str, colors: &'static [Color]) -> Self {
        Self { name, colors }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn get_ansi_escape(&self, offset: usize) -> String {
        let idx = offset.rem_euclid(self.colors.len());
        self.colors[idx].to_ansi_escape()
    }

    pub fn random_offset(&self) -> usize {
        rand::rng().random_range(0..self.colors.len())
    }

    /// a list of all of the colors in this palette
    pub fn all_ansi_escapes(&self) -> Vec<String> {
        self.colors.iter().map(Color::to_ansi_escape).collect()
    }

    pub const WHITE: Self = Self::new("White", &[ansi!(37)]);

    pub const CYAN: Self = Self::new("Cyan", &[ansi!(36)]);
    pub const YELLOW: Self = Self::new("Yellow", &[ansi!(33)]);
    pub const MAGENTA: Self = Self::new("Magenta", &[ansi!(35)]);

    // FIXME: these colors don't look good together
    pub const SYNTHWAVE: Self = Self::new(
        "Synthwave",
        &[rgb!(255, 100, 200), rgb!(100, 200, 255), rgb!(80, 70, 110)],
    );
}

pub static ALL_PALETTES: &[&Palette] = &[
    &Palette::WHITE,
    &Palette::CYAN,
    &Palette::MAGENTA,
    &Palette::YELLOW,
    &Palette::SYNTHWAVE,
];

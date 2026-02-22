use rand::RngExt;
use ratatui::style::Color as RatatuiColor;

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

    fn to_ratatui(&self) -> RatatuiColor {
        match self {
            Color::Rgb(r, g, b) => RatatuiColor::Rgb(*r, *g, *b),
            Color::Ansi(v) => match v {
                30 => RatatuiColor::Black,
                31 => RatatuiColor::Red,
                32 => RatatuiColor::Green,
                33 => RatatuiColor::Yellow,
                34 => RatatuiColor::Blue,
                35 => RatatuiColor::Magenta,
                36 => RatatuiColor::Cyan,
                37 => RatatuiColor::Gray,
                90 => RatatuiColor::DarkGray,
                91 => RatatuiColor::LightRed,
                92 => RatatuiColor::LightGreen,
                93 => RatatuiColor::LightYellow,
                94 => RatatuiColor::LightBlue,
                95 => RatatuiColor::LightMagenta,
                96 => RatatuiColor::LightCyan,
                97 => RatatuiColor::White,
                _ => RatatuiColor::Reset,
            },
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

    #[expect(dead_code)]
    pub fn get_ratatui(&self, offset: usize) -> RatatuiColor {
        let idx = offset.rem_euclid(self.colors.len());
        self.colors[idx].to_ratatui()
    }

    pub fn random_offset(&self) -> usize {
        rand::rng().random_range(0..self.colors.len())
    }

    pub fn all_ansi_escapes(&self) -> Vec<String> {
        self.colors.iter().map(Color::to_ansi_escape).collect()
    }

    pub fn all_ratatui(&self) -> Vec<RatatuiColor> {
        self.colors.iter().map(Color::to_ratatui).collect()
    }

    pub const WHITE: Self = Self::new("White", &[ansi!(37)]);

    pub const RED: Self = Self::new("Red", &[ansi!(31)]);
    pub const GREEN: Self = Self::new("Green", &[ansi!(32)]);
    pub const BLUE: Self = Self::new("Blue", &[ansi!(34)]);

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
    &Palette::RED,
    &Palette::GREEN,
    &Palette::BLUE,
    &Palette::CYAN,
    &Palette::MAGENTA,
    &Palette::YELLOW,
    &Palette::SYNTHWAVE,
];

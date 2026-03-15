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

    const BLUE: Color = rgb!(60, 60, 220);
    const BROWN: Color = rgb!(179, 117, 50);
    const CYAN: Color = rgb!(15, 245, 245);
    const DARK_RED: Color = rgb!(167, 12, 12);
    const GREEN: Color = rgb!(50, 200, 50);
    const ICE_BLUE: Color = rgb!(191, 208, 232);
    const LIGHT_GRAY: Color = rgb!(207, 207, 207);
    const MAGENTA: Color = rgb!(242, 34, 255);
    const MID_GRAY: Color = rgb!(173, 173, 173);
    const MINT: Color = rgb!(183, 240, 183);
    const NEON_GREEN: Color = rgb!(119, 255, 56);
    const NEON_PINK: Color = rgb!(255, 31, 150);
    const NEON_YELLOW: Color = rgb!(249, 255, 61);
    const ORANGE: Color = rgb!(255, 144, 31);
    const PALE_BLUE: Color = rgb!(138, 222, 255);
    const PALE_PINK: Color = rgb!(245, 169, 184);
    const PALE_YELLOW: Color = rgb!(252, 253, 190);
    const RED: Color = rgb!(244, 26, 26);
    const TEAL: Color = rgb!(49, 168, 177);
    const VIOLET: Color = rgb!(138, 43, 226);
    const WHITE: Color = rgb!(255, 255, 255);
    const YELLOW: Color = rgb!(255, 211, 25);

    const ANSI_RED: Color = ansi!(31);
    const ANSI_GREEN: Color = ansi!(32);
    const ANSI_YELLOW: Color = ansi!(33);
    const ANSI_BLUE: Color = ansi!(34);
    const ANSI_MAGENTA: Color = ansi!(35);
    const ANSI_CYAN: Color = ansi!(36);
}

pub struct Palette {
    id: &'static str,
    name: &'static str,
    colors: &'static [Color],
}

impl Palette {
    const fn new(id: &'static str, name: &'static str, colors: &'static [Color]) -> Self {
        Self { id, name, colors }
    }

    pub fn id(&self) -> &'static str {
        self.id
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

    // ansi
    pub const WHITE_ANSI: Self = Self::new("white-ansi", "White (ANSI)", &[ansi!(37)]);

    pub const RED_ANSI: Self = Self::new("red-ansi", "Red (ANSI)", &[Color::ANSI_RED]);
    pub const GREEN_ANSI: Self = Self::new("green-ansi", "Green (ANSI)", &[Color::ANSI_GREEN]);
    pub const BLUE_ANSI: Self = Self::new("blue-ansi", "Blue (ANSI)", &[Color::ANSI_BLUE]);

    pub const CYAN_ANSI: Self = Self::new("cyan-ansi", "Cyan (ANSI)", &[Color::ANSI_CYAN]);
    pub const YELLOW_ANSI: Self = Self::new("yellow-ansi", "Yellow (ANSI)", &[Color::ANSI_YELLOW]);
    pub const MAGENTA_ANSI: Self =
        Self::new("magenta-ansi", "Magenta (ANSI)", &[Color::ANSI_MAGENTA]);

    pub const RAINBOW_ANSI: Self = Self::new(
        "rainbow-ansi",
        "Rainbow (ANSI)",
        &[
            Color::ANSI_RED,
            Color::ANSI_YELLOW,
            Color::ANSI_GREEN,
            Color::ANSI_CYAN,
            Color::ANSI_BLUE,
            Color::ANSI_MAGENTA,
        ],
    );

    // single-color palettes
    pub const RED_RGB: Self = Self::new("red-rgb", "Red (RGB)", &[Color::RED]);
    pub const GREEN_RGB: Self = Self::new("green-rgb", "Green (RGB)", &[Color::GREEN]);
    pub const BLUE_RGB: Self = Self::new("blue-rgb", "Blue (RGB)", &[Color::BLUE]);
    pub const CYAN_RGB: Self = Self::new("cyan-rgb", "Cyan (RGB)", &[Color::CYAN]);
    pub const YELLOW_RGB: Self = Self::new("yellow-rgb", "Yellow (RGB)", &[Color::YELLOW]);
    pub const MAGENTA_RGB: Self = Self::new("magenta-rgb", "Magenta (RGB)", &[Color::MAGENTA]);

    // flags
    pub const FLAG_USA: Self =
        Self::new("flag-usa", "USA", &[Color::RED, Color::WHITE, Color::BLUE]);
    pub const FLAG_ITALY: Self = Self::new(
        "flag-italy",
        "Italy/Mexico",
        &[Color::GREEN, Color::WHITE, Color::RED],
    );
    pub const FLAG_UKRAINE: Self =
        Self::new("flag-ukraine", "Ukraine", &[Color::BLUE, Color::YELLOW]);
    pub const FLAG_FRANCE: Self = Self::new(
        "flag-france",
        "France",
        &[Color::BLUE, Color::WHITE, Color::RED],
    );
    pub const FLAG_TRANS: Self = Self::new(
        "flag-trans",
        "Trans Pride",
        &[
            Color::PALE_BLUE,
            Color::PALE_PINK,
            Color::WHITE,
            Color::PALE_PINK,
            Color::PALE_BLUE,
        ],
    );

    // aesthetic
    pub const SYNTHWAVE: Self = Self::new(
        "synthwave",
        "Synthwave",
        &[Color::VIOLET, Color::NEON_PINK, Color::CYAN],
    );
    pub const SUNSET: Self = Self::new(
        "sunset",
        "Sunset",
        &[
            Color::YELLOW,
            Color::ORANGE,
            Color::NEON_PINK,
            Color::MAGENTA,
            Color::VIOLET,
        ],
    );
    pub const MONOCHROME: Self = Self::new(
        "monochrome",
        "Monochrome",
        &[Color::WHITE, Color::LIGHT_GRAY, Color::MID_GRAY],
    );
    pub const RAINBOW: Self = Self::new(
        "rainbow",
        "Pride",
        &[
            Color::RED,
            Color::ORANGE,
            Color::YELLOW,
            Color::GREEN,
            Color::BLUE,
            Color::VIOLET,
        ],
    );
    pub const PASTEL: Self = Self::new(
        "pastel",
        "Pastel",
        &[
            Color::PALE_BLUE,
            Color::PALE_PINK,
            Color::MINT,
            Color::PALE_YELLOW,
        ],
    );
    pub const NEON: Self = Self::new(
        "neon",
        "Neon",
        &[
            Color::CYAN,
            Color::NEON_PINK,
            Color::NEON_GREEN,
            Color::NEON_YELLOW,
        ],
    );

    // seasonal
    pub const AUTUMN: Self = Self::new(
        "autumn",
        "Autumn",
        &[Color::DARK_RED, Color::ORANGE, Color::YELLOW, Color::BROWN],
    );
    pub const WINTER: Self = Self::new(
        "winter",
        "Winter",
        &[
            Color::WHITE,
            Color::PALE_BLUE,
            Color::LIGHT_GRAY,
            Color::ICE_BLUE,
        ],
    );
    pub const SPRING: Self = Self::new(
        "spring",
        "Spring",
        &[
            Color::NEON_GREEN,
            Color::GREEN,
            Color::MINT,
            Color::PALE_YELLOW,
        ],
    );

    // thematic
    pub const FIRE: Self = Self::new(
        "fire",
        "Fire",
        &[Color::RED, Color::YELLOW, Color::ORANGE, Color::LIGHT_GRAY],
    );
    pub const AURORA: Self = Self::new(
        "aurora",
        "Aurora",
        &[Color::GREEN, Color::TEAL, Color::VIOLET, Color::MAGENTA],
    );
}

pub static ALL_PALETTES: &[&Palette] = &[
    &Palette::WHITE_ANSI,
    &Palette::RED_ANSI,
    &Palette::GREEN_ANSI,
    &Palette::BLUE_ANSI,
    &Palette::CYAN_ANSI,
    &Palette::MAGENTA_ANSI,
    &Palette::YELLOW_ANSI,
    &Palette::RED_RGB,
    &Palette::GREEN_RGB,
    &Palette::BLUE_RGB,
    &Palette::CYAN_RGB,
    &Palette::MAGENTA_RGB,
    &Palette::YELLOW_RGB,
    &Palette::FLAG_USA,
    &Palette::FLAG_ITALY,
    &Palette::FLAG_UKRAINE,
    &Palette::FLAG_FRANCE,
    &Palette::MONOCHROME,
    &Palette::FIRE,
    &Palette::AUTUMN,
    &Palette::WINTER,
    &Palette::SPRING,
    &Palette::AURORA,
    &Palette::FLAG_TRANS,
    &Palette::SYNTHWAVE,
    &Palette::PASTEL,
    &Palette::NEON,
    &Palette::SUNSET,
    &Palette::RAINBOW,
    &Palette::RAINBOW_ANSI,
];

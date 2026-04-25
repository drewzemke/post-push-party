use rand::RngExt;
use ratatui::style::Color as RatatuiColor;
use tixel::{AnsiColor, Color};

fn tixel_to_ratatui(color: &Color) -> RatatuiColor {
    match color {
        Color::Rgb(r, g, b) => RatatuiColor::Rgb(*r, *g, *b),
        Color::Ansi(ansi) => match ansi {
            AnsiColor::Black => RatatuiColor::Black,
            AnsiColor::Red => RatatuiColor::Red,
            AnsiColor::Green => RatatuiColor::Green,
            AnsiColor::Yellow => RatatuiColor::Yellow,
            AnsiColor::Blue => RatatuiColor::Blue,
            AnsiColor::Magenta => RatatuiColor::Magenta,
            AnsiColor::Cyan => RatatuiColor::Cyan,
            AnsiColor::White => RatatuiColor::Gray,
            AnsiColor::BrightBlack => RatatuiColor::DarkGray,
            AnsiColor::BrightRed => RatatuiColor::LightRed,
            AnsiColor::BrightGreen => RatatuiColor::LightGreen,
            AnsiColor::BrightYellow => RatatuiColor::LightYellow,
            AnsiColor::BrightBlue => RatatuiColor::LightBlue,
            AnsiColor::BrightMagenta => RatatuiColor::LightMagenta,
            AnsiColor::BrightCyan => RatatuiColor::LightCyan,
            AnsiColor::BrightWhite => RatatuiColor::White,
        },
    }
}

const BLUE: Color = Color::Rgb(60, 60, 220);
const BROWN: Color = Color::Rgb(179, 117, 50);
const CYAN: Color = Color::Rgb(15, 245, 245);
const DARK_RED: Color = Color::Rgb(167, 12, 12);
const GREEN: Color = Color::Rgb(50, 200, 50);
const ICE_BLUE: Color = Color::Rgb(191, 208, 232);
const LIGHT_GRAY: Color = Color::Rgb(207, 207, 207);
const MAGENTA: Color = Color::Rgb(242, 34, 255);
const MID_GRAY: Color = Color::Rgb(173, 173, 173);
const MINT: Color = Color::Rgb(183, 240, 183);
const NEON_GREEN: Color = Color::Rgb(119, 255, 56);
const NEON_PINK: Color = Color::Rgb(255, 31, 150);
const NEON_YELLOW: Color = Color::Rgb(249, 255, 61);
const ORANGE: Color = Color::Rgb(255, 144, 31);
const PALE_BLUE: Color = Color::Rgb(138, 222, 255);
const PALE_PINK: Color = Color::Rgb(245, 169, 184);
const PALE_YELLOW: Color = Color::Rgb(252, 253, 190);
const RED: Color = Color::Rgb(244, 26, 26);
const TEAL: Color = Color::Rgb(49, 168, 177);
const VIOLET: Color = Color::Rgb(138, 43, 226);
const WHITE: Color = Color::Rgb(255, 255, 255);
const YELLOW: Color = Color::Rgb(255, 211, 25);

const ANSI_RED: Color = Color::Ansi(AnsiColor::Red);
const ANSI_GREEN: Color = Color::Ansi(AnsiColor::Green);
const ANSI_YELLOW: Color = Color::Ansi(AnsiColor::Yellow);
const ANSI_BLUE: Color = Color::Ansi(AnsiColor::Blue);
const ANSI_MAGENTA: Color = Color::Ansi(AnsiColor::Magenta);
const ANSI_CYAN: Color = Color::Ansi(AnsiColor::Cyan);
const ANSI_WHITE: Color = Color::Ansi(AnsiColor::White);

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

    pub fn get_color(&self, offset: usize) -> Color {
        let idx = offset.rem_euclid(self.colors.len());
        self.colors[idx]
    }

    pub fn random_offset(&self) -> usize {
        rand::rng().random_range(0..self.colors.len())
    }

    pub fn colors(&self) -> &[Color] {
        self.colors
    }

    pub fn all_ratatui(&self) -> Vec<RatatuiColor> {
        self.colors.iter().map(tixel_to_ratatui).collect()
    }

    // ansi
    pub const WHITE_ANSI: Self = Self::new("white-ansi", "White (ANSI)", &[ANSI_WHITE]);

    pub const RED_ANSI: Self = Self::new("red-ansi", "Red (ANSI)", &[ANSI_RED]);
    pub const GREEN_ANSI: Self = Self::new("green-ansi", "Green (ANSI)", &[ANSI_GREEN]);
    pub const BLUE_ANSI: Self = Self::new("blue-ansi", "Blue (ANSI)", &[ANSI_BLUE]);

    pub const CYAN_ANSI: Self = Self::new("cyan-ansi", "Cyan (ANSI)", &[ANSI_CYAN]);
    pub const YELLOW_ANSI: Self = Self::new("yellow-ansi", "Yellow (ANSI)", &[ANSI_YELLOW]);
    pub const MAGENTA_ANSI: Self = Self::new("magenta-ansi", "Magenta (ANSI)", &[ANSI_MAGENTA]);

    pub const RAINBOW_ANSI: Self = Self::new(
        "rainbow-ansi",
        "Rainbow (ANSI)",
        &[
            ANSI_RED,
            ANSI_YELLOW,
            ANSI_GREEN,
            ANSI_CYAN,
            ANSI_BLUE,
            ANSI_MAGENTA,
        ],
    );

    // single-color palettes
    pub const RED_RGB: Self = Self::new("red-rgb", "Red (RGB)", &[RED]);
    pub const GREEN_RGB: Self = Self::new("green-rgb", "Green (RGB)", &[GREEN]);
    pub const BLUE_RGB: Self = Self::new("blue-rgb", "Blue (RGB)", &[BLUE]);
    pub const CYAN_RGB: Self = Self::new("cyan-rgb", "Cyan (RGB)", &[CYAN]);
    pub const YELLOW_RGB: Self = Self::new("yellow-rgb", "Yellow (RGB)", &[YELLOW]);
    pub const MAGENTA_RGB: Self = Self::new("magenta-rgb", "Magenta (RGB)", &[MAGENTA]);

    // flags
    pub const FLAG_USA: Self = Self::new("flag-usa", "USA", &[RED, WHITE, BLUE]);
    pub const FLAG_ITALY: Self = Self::new("flag-italy", "Italy/Mexico", &[GREEN, WHITE, RED]);
    pub const FLAG_UKRAINE: Self = Self::new("flag-ukraine", "Ukraine", &[BLUE, YELLOW]);
    pub const FLAG_FRANCE: Self = Self::new("flag-france", "France", &[BLUE, WHITE, RED]);
    pub const FLAG_TRANS: Self = Self::new(
        "flag-trans",
        "Trans Pride",
        &[PALE_BLUE, PALE_PINK, WHITE, PALE_PINK, PALE_BLUE],
    );

    // aesthetic
    pub const SYNTHWAVE: Self = Self::new("synthwave", "Synthwave", &[VIOLET, NEON_PINK, CYAN]);
    pub const SUNSET: Self = Self::new(
        "sunset",
        "Sunset",
        &[YELLOW, ORANGE, NEON_PINK, MAGENTA, VIOLET],
    );
    pub const MONOCHROME: Self =
        Self::new("monochrome", "Monochrome", &[WHITE, LIGHT_GRAY, MID_GRAY]);
    pub const RAINBOW: Self = Self::new(
        "rainbow",
        "Pride",
        &[RED, ORANGE, YELLOW, GREEN, BLUE, VIOLET],
    );
    pub const PASTEL: Self = Self::new(
        "pastel",
        "Pastel",
        &[PALE_BLUE, PALE_PINK, MINT, PALE_YELLOW],
    );
    pub const NEON: Self = Self::new("neon", "Neon", &[CYAN, NEON_PINK, NEON_GREEN, NEON_YELLOW]);

    // seasonal
    pub const AUTUMN: Self = Self::new("autumn", "Autumn", &[DARK_RED, ORANGE, YELLOW, BROWN]);
    pub const WINTER: Self = Self::new(
        "winter",
        "Winter",
        &[WHITE, PALE_BLUE, LIGHT_GRAY, ICE_BLUE],
    );
    pub const SPRING: Self = Self::new("spring", "Spring", &[NEON_GREEN, GREEN, MINT, PALE_YELLOW]);

    // thematic
    pub const FIRE: Self = Self::new("fire", "Fire", &[RED, YELLOW, ORANGE, LIGHT_GRAY]);
    pub const AURORA: Self = Self::new("aurora", "Aurora", &[GREEN, TEAL, VIOLET, MAGENTA]);
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

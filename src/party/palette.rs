use rand::RngExt;

/// generates a static str escape sequence for an rbg color
macro_rules! rgb {
    ($r: literal, $g: literal, $b: literal) => {
        concat!(
            "\x1b[38;2;",
            stringify!($r),
            ";",
            stringify!($g),
            ";",
            stringify!($b),
            "m"
        )
    };
}

/// generates a static str escape sequence for an ansi color
macro_rules! ansi {
    ($code: literal) => {
        concat!("\x1b[", stringify!($code), "m")
    };
}

pub struct Palette {
    name: &'static str,
    colors: &'static [&'static str],
}

impl Palette {
    pub const fn new(name: &'static str, colors: &'static [&'static str]) -> Self {
        Self { name, colors }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn get(&self, offset: usize) -> &'static str {
        let idx = offset.rem_euclid(self.colors.len());
        self.colors[idx]
    }

    pub fn random_offset(&self) -> usize {
        rand::rng().random_range(0..self.colors.len())
    }

    /// a list of all of the colors in this palette
    pub fn colors(&self) -> &[&str] {
        self.colors
    }

    pub const WHITE: Self = Self::new("White", &[ansi!(37)]);

    pub const CYAN: Self = Self::new("Cyan", &[ansi!(36)]);
    pub const YELLOW: Self = Self::new("Yellow", &[ansi!(35)]);
    pub const MAGENTA: Self = Self::new("Magenta", &[ansi!(33)]);

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

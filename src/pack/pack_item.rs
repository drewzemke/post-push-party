use crate::{
    pack::Rarity,
    party::{FIREWORKS, Palette, Party},
    state::State,
};

/// what can be in a pack
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PackItem {
    PaletteUnlock {
        party_id: &'static str,
        palette_id: &'static str,
        rarity: Rarity,
    },
    PointBundle {
        points: u64,
        rarity: Rarity,
    },
    // GameToken (GameType)   // soon(ish)!
}

const COMMON_POINTS: u64 = 25;
const RARE_POINTS: u64 = 100;
const EPIC_POINTS: u64 = 400;
const LEGENDARY_POINTS: u64 = 1600;

const COMMON_PALETTES: &[Palette] = &[
    Palette::RED_ANSI,
    Palette::GREEN_ANSI,
    Palette::BLUE_ANSI,
    Palette::CYAN_ANSI,
    Palette::MAGENTA_ANSI,
    Palette::YELLOW_ANSI,
    Palette::RED_RGB,
    Palette::GREEN_RGB,
    Palette::BLUE_RGB,
    Palette::CYAN_RGB,
    Palette::MAGENTA_RGB,
    Palette::YELLOW_RGB,
];

const RARE_PALETTES: &[Palette] = &[
    Palette::FLAG_USA,
    Palette::FLAG_ITALY,
    Palette::FLAG_UKRAINE,
    Palette::FLAG_FRANCE,
    Palette::FLAG_TRANS,
    Palette::MONOCHROME,
    Palette::AUTUMN,
    Palette::WINTER,
    Palette::SPRING,
    Palette::FIRE,
];

const EPIC_PALETTES: &[Palette] = &[
    Palette::AURORA,
    Palette::NEON,
    Palette::PASTEL,
    Palette::RAINBOW,
    Palette::RAINBOW_ANSI,
    Palette::SUNSET,
    Palette::SYNTHWAVE,
];

impl PackItem {
    pub fn rarity(&self) -> Rarity {
        match self {
            PackItem::PaletteUnlock { rarity, .. } => *rarity,
            PackItem::PointBundle { rarity, .. } => *rarity,
        }
    }

    /// all items of a given rarity that are available to be opened
    /// in packs based on current state. in particular, this excludes
    /// all already-unlocked palettes
    pub fn available_items(rarity: Rarity, state: &State) -> Vec<Self> {
        match rarity {
            Rarity::Common => Self::common_items(state),
            Rarity::Rare => Self::rare_items(state),
            Rarity::Epic => Self::epic_items(state),
            Rarity::Legendary => Self::legendary_items(state),
        }
    }

    pub(super) fn common_items(state: &State) -> Vec<Self> {
        state
            .unlocked_parties()
            .filter(|party| party.supports_color() && party.id() != FIREWORKS.id())
            .flat_map(|party| {
                COMMON_PALETTES
                    .iter()
                    .filter(|palette| !state.is_palette_unlocked(party.id(), palette.id()))
                    .map(|palette| Self::PaletteUnlock {
                        party_id: party.id(),
                        palette_id: palette.id(),
                        rarity: Rarity::Common,
                    })
            })
            .chain([Self::PointBundle {
                points: COMMON_POINTS,
                rarity: Rarity::Common,
            }])
            .collect()
    }

    fn rare_items(state: &State) -> Vec<Self> {
        let iter = state
            .unlocked_parties()
            .filter(|party| party.supports_color() && party.id() != FIREWORKS.id())
            .flat_map(|party| {
                RARE_PALETTES
                    .iter()
                    .filter(|palette| !state.is_palette_unlocked(party.id(), palette.id()))
                    .map(|palette| Self::PaletteUnlock {
                        party_id: party.id(),
                        palette_id: palette.id(),
                        rarity: Rarity::Rare,
                    })
            })
            .chain([Self::PointBundle {
                points: RARE_POINTS,
                rarity: Rarity::Rare,
            }]);

        if state.is_party_unlocked(FIREWORKS.id()) {
            iter.chain(
                COMMON_PALETTES
                    .iter()
                    .filter(|palette| !state.is_palette_unlocked(FIREWORKS.id(), palette.id()))
                    .map(|palette| Self::PaletteUnlock {
                        party_id: FIREWORKS.id(),
                        palette_id: palette.id(),
                        rarity: Rarity::Rare,
                    }),
            )
            .collect()
        } else {
            iter.collect()
        }
    }

    fn epic_items(state: &State) -> Vec<Self> {
        let iter = state
            .unlocked_parties()
            .filter(|party| party.supports_color() && party.id() != FIREWORKS.id())
            .flat_map(|party| {
                EPIC_PALETTES
                    .iter()
                    .filter(|palette| !state.is_palette_unlocked(party.id(), palette.id()))
                    .map(|palette| Self::PaletteUnlock {
                        party_id: party.id(),
                        palette_id: palette.id(),
                        rarity: Rarity::Epic,
                    })
            })
            .chain([Self::PointBundle {
                points: EPIC_POINTS,
                rarity: Rarity::Epic,
            }]);

        if state.is_party_unlocked(FIREWORKS.id()) {
            iter.chain(
                RARE_PALETTES
                    .iter()
                    .filter(|palette| !state.is_palette_unlocked(FIREWORKS.id(), palette.id()))
                    .map(|palette| Self::PaletteUnlock {
                        party_id: FIREWORKS.id(),
                        palette_id: palette.id(),
                        rarity: Rarity::Epic,
                    }),
            )
            .collect()
        } else {
            iter.collect()
        }
    }

    fn legendary_items(state: &State) -> Vec<Self> {
        if state.is_party_unlocked(FIREWORKS.id()) {
            EPIC_PALETTES
                .iter()
                .filter(|palette| !state.is_palette_unlocked(FIREWORKS.id(), palette.id()))
                .map(|palette| Self::PaletteUnlock {
                    party_id: FIREWORKS.id(),
                    palette_id: palette.id(),
                    rarity: Rarity::Legendary,
                })
                .chain([Self::PointBundle {
                    points: LEGENDARY_POINTS,
                    rarity: Rarity::Legendary,
                }])
                .collect()
        } else {
            Vec::from([Self::PointBundle {
                points: LEGENDARY_POINTS,
                rarity: Rarity::Legendary,
            }])
        }
    }

    pub fn apply(&self, state: &mut State) {
        match self {
            PackItem::PaletteUnlock {
                party_id,
                palette_id,
                ..
            } => {
                state.unlock_palette(party_id, palette_id);
            }
            // directly updating party points here, because we don't
            // want to trigger the lifetime points mechanism this way
            PackItem::PointBundle { points, .. } => state.party_points += *points,
        };
    }
}

use rand::{RngExt, seq::IndexedRandom};
use serde::{Deserialize, Serialize};

use crate::{
    party::{FIREWORKS, Palette, Party},
    state::State,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Pack {
    Basic,
    Premium, // others...?
}

pub const ALL_PACKS: &[Pack] = &[Pack::Basic, Pack::Premium];

impl Pack {
    pub fn cost(&self) -> u64 {
        match self {
            Pack::Basic => 1_000,
            Pack::Premium => 10_000,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Pack::Basic => "Basic Pack",
            Pack::Premium => "Premium Pack",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Pack::Basic => "A pack with mostly common items and at least one of higher-rarity.",
            Pack::Premium => "A pack with only three items but of higher-rarity.",
        }
    }

    fn template(&self) -> PackTemplate {
        match self {
            Pack::Basic => BASIC_PACK_TEMPLATE,
            Pack::Premium => PREMIUM_PACK_TEMPLATE,
        }
    }

    /// performs the pack opening algorithm, adding the obtained items to user's state.
    /// returns the opened items
    pub fn open(&self, state: &mut State) -> Vec<PackItem> {
        self.open_with_upgrader(state, &mut RngUpgrader::new(rand::rng()))
    }

    fn open_with_upgrader(
        &self,
        state: &mut State,
        upgrader: &mut impl RarityUpgrader,
    ) -> Vec<PackItem> {
        // get template for this pack
        let template = self.template();

        // upgrade the template
        let upgraded_rarities = template.upgrade(upgrader);

        // iteratively choose items based on state
        upgraded_rarities
            .into_iter()
            .map(|rarity| {
                let item = Self::resolve_item(rarity, state);
                // NOTE: applying to state in this loop to guarantee that
                // future iterations don't resolve the same item
                item.apply(state);
                item
            })
            .collect()
    }

    /// chooses an item of the given rarity that the player has not yet unlocked
    fn resolve_item(rarity: Rarity, state: &State) -> PackItem {
        let mut rng = rand::rng();
        let items = PackItem::available_items(rarity, state);
        *items
            .choose(&mut rng)
            .expect("there should always be at least one unlockable item")
    }
}

/// the base rarity of items in a pack
#[derive(Debug, Copy, Clone, PartialEq)]
struct PackTemplate(&'static [Rarity]);

const BASIC_PACK_TEMPLATE: PackTemplate = PackTemplate(&[
    Rarity::Common,
    Rarity::Common,
    Rarity::Common,
    Rarity::Common,
    Rarity::Rare,
]);

const PREMIUM_PACK_TEMPLATE: PackTemplate =
    PackTemplate(&[Rarity::Common, Rarity::Rare, Rarity::Epic]);

impl PackTemplate {
    /// iteratively upgrades the elements of the template
    fn upgrade(&self, upgrader: &mut impl RarityUpgrader) -> Vec<Rarity> {
        self.0
            .iter()
            .map(|rarity| {
                let mut rarity = *rarity;
                while upgrader.should_upgrade(&rarity) {
                    if let Some(new) = rarity.upgrade() {
                        rarity = new;
                    } else {
                        break;
                    }
                }
                rarity
            })
            .collect()
    }
}

/// how uncommon a pack item is
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Rarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

impl Rarity {
    fn upgrade(&self) -> Option<Self> {
        match self {
            Self::Common => Some(Self::Rare),
            Self::Rare => Some(Self::Epic),
            Self::Epic => Some(Self::Legendary),
            Self::Legendary => None,
        }
    }
}

/// what can be in a pack
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PackItem {
    PaletteUnlock {
        party_id: &'static str,
        palette_name: &'static str,
        rarity: Rarity,
    },
    PointBundle {
        points: u64,
        rarity: Rarity,
    },
    // GameToken (GameType)   // soon(ish)!
}

const COMMON_POINTS: u64 = 100;
const RARE_POINTS: u64 = 400;
const EPIC_POINTS: u64 = 1_000;
const LEGENDARY_POINTS: u64 = 10_000;

const COMMON_PALETTES: &[Palette] = &[Palette::RED, Palette::GREEN, Palette::BLUE];

const RARE_PALETTES: &[Palette] = &[Palette::CYAN, Palette::MAGENTA, Palette::YELLOW];

const EPIC_PALETTES: &[Palette] = &[Palette::SYNTHWAVE];

impl PackItem {
    #[cfg(test)]
    fn rarity(&self) -> Rarity {
        match self {
            PackItem::PaletteUnlock { rarity, .. } => *rarity,
            PackItem::PointBundle { rarity, .. } => *rarity,
        }
    }

    /// all items of a given rarity that are available to be opened
    /// in packs based on current state. in particular, this excludes
    /// all already-unlocked palettes
    fn available_items(rarity: Rarity, state: &State) -> Vec<Self> {
        match rarity {
            Rarity::Common => Self::common_items(state),
            Rarity::Rare => Self::rare_items(state),
            Rarity::Epic => Self::epic_items(state),
            Rarity::Legendary => Self::legendary_items(state),
        }
    }

    fn common_items(state: &State) -> Vec<Self> {
        state
            .unlocked_parties()
            .filter(|party| party.id() != FIREWORKS.id())
            .flat_map(|party| {
                COMMON_PALETTES
                    .iter()
                    .filter(|palette| !state.is_palette_unlocked(party.id(), palette.name()))
                    .map(|palette| Self::PaletteUnlock {
                        party_id: party.id(),
                        palette_name: palette.name(),
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
            .filter(|party| party.id() != FIREWORKS.id())
            .flat_map(|party| {
                RARE_PALETTES
                    .iter()
                    .filter(|palette| !state.is_palette_unlocked(party.id(), palette.name()))
                    .map(|palette| Self::PaletteUnlock {
                        party_id: party.id(),
                        palette_name: palette.name(),
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
                    .filter(|palette| !state.is_palette_unlocked(FIREWORKS.id(), palette.name()))
                    .map(|palette| Self::PaletteUnlock {
                        party_id: FIREWORKS.id(),
                        palette_name: palette.name(),
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
            .filter(|party| party.id() != FIREWORKS.id())
            .flat_map(|party| {
                EPIC_PALETTES
                    .iter()
                    .filter(|palette| !state.is_palette_unlocked(party.id(), palette.name()))
                    .map(|palette| Self::PaletteUnlock {
                        party_id: party.id(),
                        palette_name: palette.name(),
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
                    .filter(|palette| !state.is_palette_unlocked(FIREWORKS.id(), palette.name()))
                    .map(|palette| Self::PaletteUnlock {
                        party_id: FIREWORKS.id(),
                        palette_name: palette.name(),
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
                .filter(|palette| !state.is_palette_unlocked(FIREWORKS.id(), palette.name()))
                .map(|palette| Self::PaletteUnlock {
                    party_id: FIREWORKS.id(),
                    palette_name: palette.name(),
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
                palette_name,
                ..
            } => {
                state.unlock_palette(party_id, palette_name);
            }
            // directly updating party points here, because we don't
            // want to trigger the lifetime points mechanism this way
            PackItem::PointBundle { points, .. } => state.party_points += *points,
        };
    }
}

trait RarityUpgrader {
    fn should_upgrade(&mut self, rarity: &Rarity) -> bool;
}

struct RngUpgrader<R: rand::Rng> {
    rng: R,
}

impl<R: rand::Rng> RngUpgrader<R> {
    fn new(rng: R) -> Self {
        Self { rng }
    }
}

// for the basic pack template (CCCCR), the probabilities below
// yield an item distribution of (approx)
//   72% C, 22% R, 5% E, 1%
//   epic every 4 packs
//   legendary every 18 packs
//   one in four packs contains epic/legendary
const COMMON_TO_RARE_PROB: f64 = 0.10;
const RARE_TO_EPIC_PROB: f64 = 0.20;
const EPIC_TO_LEGENDARY_PROB: f64 = 0.20;

impl<R: rand::Rng> RarityUpgrader for RngUpgrader<R> {
    fn should_upgrade(&mut self, rarity: &Rarity) -> bool {
        let roll = self.rng.random_range(0.0..1.0);
        match rarity {
            Rarity::Common => roll < COMMON_TO_RARE_PROB,
            Rarity::Rare => roll < RARE_TO_EPIC_PROB,
            Rarity::Epic => roll < EPIC_TO_LEGENDARY_PROB,
            Rarity::Legendary => false,
        }
    }
}

#[cfg(test)]
struct MockUpgrader<'a> {
    will_upgrade: &'a [Rarity],
}

#[cfg(test)]
impl<'a> MockUpgrader<'a> {
    fn new(will_upgrade: &'a [Rarity]) -> Self {
        Self { will_upgrade }
    }
}

#[cfg(test)]
impl<'a> RarityUpgrader for MockUpgrader<'a> {
    fn should_upgrade(&mut self, rarity: &Rarity) -> bool {
        self.will_upgrade.contains(rarity)
    }
}

#[cfg(test)]
mod tests {
    use crate::party::BASE;

    use super::*;

    #[test]
    fn no_upgrades_to_template() {
        let template = PackTemplate(&[
            Rarity::Common,
            Rarity::Rare,
            Rarity::Epic,
            Rarity::Legendary,
        ]);

        let mut upgrader = MockUpgrader::new(&[]);

        let upgraded = template.upgrade(&mut upgrader);

        assert_eq!(
            upgraded,
            &[
                Rarity::Common,
                Rarity::Rare,
                Rarity::Epic,
                Rarity::Legendary,
            ]
        );
    }

    #[test]
    fn test_upgrade_only_rares() {
        let template = PackTemplate(&[
            Rarity::Common,
            Rarity::Rare,
            Rarity::Epic,
            Rarity::Legendary,
        ]);

        let mut upgrader = MockUpgrader::new(&[Rarity::Rare]);

        let upgraded = template.upgrade(&mut upgrader);

        assert_eq!(
            &upgraded,
            &[
                Rarity::Common,
                Rarity::Epic,
                Rarity::Epic,
                Rarity::Legendary,
            ]
        );
    }

    #[test]
    fn test_full_upgrade() {
        let template = PackTemplate(&[
            Rarity::Common,
            Rarity::Rare,
            Rarity::Epic,
            Rarity::Legendary,
        ]);

        let mut upgrader = MockUpgrader::new(&[
            Rarity::Common,
            Rarity::Rare,
            Rarity::Epic,
            Rarity::Legendary,
        ]);

        let upgraded = template.upgrade(&mut upgrader);

        assert_eq!(
            upgraded,
            &[
                Rarity::Legendary,
                Rarity::Legendary,
                Rarity::Legendary,
                Rarity::Legendary,
            ]
        );
    }

    #[test]
    fn test_resolve_items_basic() {
        // state starts with one party unlocked, so there
        // are things that can be unlocked of all rarities
        let state = State::default();

        for rarity in [
            Rarity::Common,
            Rarity::Rare,
            Rarity::Epic,
            Rarity::Legendary,
        ] {
            let item = Pack::resolve_item(rarity, &state);
            assert_eq!(item.rarity(), rarity);
        }
    }

    #[test]
    fn test_resolve_items_no_repeats() {
        // state with all but one common unlocked
        let mut state = State::default();

        let target = PackItem::PaletteUnlock {
            party_id: BASE.id(),
            palette_name: Palette::MAGENTA.name(),
            rarity: Rarity::Common,
        };
        let commons = PackItem::common_items(&state);

        for item in commons {
            if item == target {
                continue;
            }

            item.apply(&mut state);
        }

        // resolve one common item
        // it should be the one we skipped OR points
        let item = Pack::resolve_item(Rarity::Common, &state);
        assert!(item == target || matches!(item, PackItem::PointBundle { .. }));
    }

    #[test]
    fn resolve_items_grants_points_if_everything_unlocked() {
        // state with all palettes common unlocked
        let mut state = State::default();

        let commons = PackItem::common_items(&state);

        for item in commons {
            if matches!(item, PackItem::PointBundle { .. }) {
                continue;
            }

            item.apply(&mut state);
        }

        // resolve one more common item

        // it should be the points
        let item = Pack::resolve_item(Rarity::Common, &state);
        assert!(matches!(item, PackItem::PointBundle { .. }));
    }
}

use rand::seq::{IndexedRandom, SliceRandom};

use crate::state::State;

mod pack_item;
mod rarity;

use rarity::{RarityUpgrader, RngUpgrader};

pub use pack_item::PackItem;
pub use rarity::{PackTemplate, Rarity};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pack {
    Basic,
    Premium, // others...?
}

pub const ALL_PACKS: &[Pack] = &[Pack::Basic, Pack::Premium];

pub const BASIC_PACK_TEMPLATE: PackTemplate = PackTemplate(&[
    Rarity::Common,
    Rarity::Common,
    Rarity::Common,
    Rarity::Common,
    Rarity::Rare,
]);

pub const PREMIUM_PACK_TEMPLATE: PackTemplate =
    PackTemplate(&[Rarity::Common, Rarity::Rare, Rarity::Epic]);

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
        let mut upgraded_rarities = template.upgrade(upgrader);

        // shuffle rarities for extra fun and surprise
        upgraded_rarities.shuffle(&mut rand::rng());

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

#[cfg(test)]
mod tests {
    use crate::party::{BASE, Palette, Party};

    use super::*;

    struct MockUpgrader<'a> {
        will_upgrade: &'a [Rarity],
    }

    impl<'a> MockUpgrader<'a> {
        fn new(will_upgrade: &'a [Rarity]) -> Self {
            Self { will_upgrade }
        }
    }

    impl<'a> RarityUpgrader for MockUpgrader<'a> {
        fn should_upgrade(&mut self, rarity: &Rarity) -> bool {
            self.will_upgrade.contains(rarity)
        }
    }

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
}

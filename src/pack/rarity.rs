use rand::RngExt;

/// how uncommon a pack item is
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Rarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

impl Rarity {
    pub fn upgrade(&self) -> Option<Self> {
        match self {
            Self::Common => Some(Self::Rare),
            Self::Rare => Some(Self::Epic),
            Self::Epic => Some(Self::Legendary),
            Self::Legendary => None,
        }
    }

    /// gets colors in HSL
    pub fn color(&self) -> (f32, f32, f32) {
        match self {
            Rarity::Common => (0., 0., 0.55),
            Rarity::Rare => (240., 1., 0.7),
            Rarity::Epic => (280., 1., 0.55),
            Rarity::Legendary => (45., 1., 0.50),
        }
    }
}

/// the base rarity of items in a pack
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PackTemplate(pub &'static [Rarity]);

impl PackTemplate {
    /// iteratively upgrades the elements of the template
    pub fn upgrade(&self, upgrader: &mut impl RarityUpgrader) -> Vec<Rarity> {
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

pub trait RarityUpgrader {
    fn should_upgrade(&mut self, rarity: &Rarity) -> bool;
}

pub struct RngUpgrader<R: rand::Rng> {
    rng: R,
}

impl<R: rand::Rng> RngUpgrader<R> {
    pub fn new(rng: R) -> Self {
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

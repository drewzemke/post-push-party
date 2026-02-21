use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Pack {
    Basic, // others...
}

pub const ALL_PACKS: &[Pack] = &[Pack::Basic];

impl Pack {
    pub fn cost(&self) -> u64 {
        match self {
            Pack::Basic => 1_000,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Pack::Basic => "Basic Pack",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Pack::Basic => "A pack with mostly common items and at least one of higher-rarity.",
        }
    }
}

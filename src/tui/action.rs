/// input actions, decoupled from specific keys
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
    Select,
    Back,
    Tab(usize),
    NextTab,
    PrevTab,
    Palette,
    Quit,
}

pub const NUM_TABS: usize = 4;

/// navigation targets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Store(StoreRoute),
    Party,
    Packs,
    PackReveal,
    Games,
}

impl Default for Route {
    fn default() -> Self {
        Route::Store(StoreRoute::default())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StoreRoute {
    #[default]
    Grid,
    Upgrades,
    Bonuses,
    Packs,
}

impl Route {
    pub fn tab_index(&self) -> usize {
        match self {
            Route::Store(_) => 0,
            Route::Party => 1,
            Route::Packs => 2,
            Route::PackReveal => 2,
            Route::Games => 3,
        }
    }
}

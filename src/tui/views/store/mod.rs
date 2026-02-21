mod bonuses;
mod grid;
mod packs;
mod upgrades;

use ratatui::prelude::*;

use crate::state::State;
use crate::tui::action::{Action, Route, StoreRoute};
use crate::tui::views::store::packs::PacksView;
use crate::tui::views::{View, ViewResult};

pub use bonuses::BonusesView;
pub use grid::GridView;
pub use upgrades::UpgradesView;

pub struct StoreView {
    route: StoreRoute,
    grid: GridView,
    upgrades: UpgradesView,
    bonuses: BonusesView,
    packs: PacksView,
}

impl Default for StoreView {
    fn default() -> Self {
        Self {
            route: StoreRoute::Grid,
            grid: GridView::default(),
            upgrades: UpgradesView::default(),
            bonuses: BonusesView::default(),
            packs: PacksView::default(),
        }
    }
}

impl StoreView {
    pub fn set_route(&mut self, route: StoreRoute) {
        self.route = route;
    }

    fn current_view(&self) -> &dyn View {
        match self.route {
            StoreRoute::Grid => &self.grid,
            StoreRoute::Upgrades => &self.upgrades,
            StoreRoute::Bonuses => &self.bonuses,
            StoreRoute::Packs => &self.packs,
        }
    }

    fn current_view_mut(&mut self) -> &mut dyn View {
        match self.route {
            StoreRoute::Grid => &mut self.grid,
            StoreRoute::Upgrades => &mut self.upgrades,
            StoreRoute::Bonuses => &mut self.bonuses,
            StoreRoute::Packs => &mut self.packs,
        }
    }
}

impl View for StoreView {
    fn render(&self, frame: &mut Frame, area: Rect, state: &State, tick: u32) {
        self.current_view().render(frame, area, state, tick);
    }

    fn handle(&mut self, action: Action, state: &mut State) -> ViewResult {
        // handle Back action to return to grid from sub-pages
        if action == Action::Back && self.route != StoreRoute::Grid {
            self.route = StoreRoute::Grid;
            return ViewResult::Redraw;
        }

        let result = self.current_view_mut().handle(action, state);

        // if sub-view navigates within store, update our route
        if let ViewResult::Navigate(Route::Store(sub_route)) = result {
            self.route = sub_route;
            return ViewResult::Redraw;
        }

        result
    }

    fn key_hints(&self) -> Vec<(&'static str, &'static str)> {
        self.current_view().key_hints()
    }
}

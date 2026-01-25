use ratatui::prelude::*;

use crate::state::State;
use crate::tui::action::{Action, Route, StoreRoute};
use crate::tui::views::{View, ViewResult};
use crate::tui::widgets::Card;

const GRID_ITEMS: [(StoreRoute, &str, &str); 4] = [
    (
        StoreRoute::Upgrades,
        "Party Upgrades",
        "Make your party more fancy, set colors.",
    ),
    (
        StoreRoute::Bonuses,
        "Bonuses",
        "Unlock ways to earn more points.",
    ),
    (
        StoreRoute::Grid, // placeholder - packs not implemented yet
        "Packs",
        "Buy packs which contain upgrades, points, games.",
    ),
    (
        StoreRoute::Grid, // placeholder - games not implemented yet
        "Games",
        "Spend points to unlock more attempts at mini-games.",
    ),
];

pub struct GridView {
    selection: usize,
}

impl Default for GridView {
    fn default() -> Self {
        Self { selection: 0 }
    }
}

impl View for GridView {
    fn render(&self, frame: &mut Frame, area: Rect, _state: &State) {
        // 2x2 grid layout
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .margin(1)
            .split(area);

        let top_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(rows[0]);

        let bottom_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(rows[1]);

        let cells = [top_cols[0], top_cols[1], bottom_cols[0], bottom_cols[1]];

        for (i, &(_, title, desc)) in GRID_ITEMS.iter().enumerate() {
            let card = Card::new()
                .title(title)
                .content(vec![Line::from(desc)])
                .selected(i == self.selection);
            frame.render_widget(card, cells[i]);
        }
    }

    fn handle(&mut self, action: Action, _state: &mut State) -> ViewResult {
        match action {
            Action::Up => {
                if self.selection >= 2 {
                    self.selection -= 2;
                }
                ViewResult::Redraw
            }
            Action::Down => {
                if self.selection < 2 {
                    self.selection += 2;
                }
                ViewResult::Redraw
            }
            Action::Left => {
                if self.selection % 2 == 1 {
                    self.selection -= 1;
                }
                ViewResult::Redraw
            }
            Action::Right => {
                if self.selection % 2 == 0 {
                    self.selection += 1;
                }
                ViewResult::Redraw
            }
            Action::Select => {
                let (route, _, _) = GRID_ITEMS[self.selection];
                if route == StoreRoute::Grid {
                    ViewResult::Message("Coming soon...".to_string())
                } else {
                    ViewResult::Navigate(Route::Store(route))
                }
            }
            Action::Tab(i) => ViewResult::Navigate(match i {
                0 => Route::Store(Default::default()),
                1 => Route::Party,
                2 => Route::Packs,
                _ => Route::Games,
            }),
            Action::Quit => ViewResult::Exit,
            _ => ViewResult::None,
        }
    }

    fn key_hints(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("↑↓←→", "select"),
            ("Enter", "open"),
            ("1-4", "tab"),
            ("q", "quit"),
        ]
    }
}

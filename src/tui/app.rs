use ratatui::prelude::*;

use crate::state::State;
use crate::storage::DbConnection;
use crate::tui::views::MessageType;

use super::action::{Action, Route};
use super::views::games::GamesView;
use super::views::packs::PacksView;
use super::views::party::PartyView;
use super::views::store::StoreView;
use super::views::{View, ViewResult};
use super::widgets::{render_footer, render_header};

pub struct App<'a> {
    route: Route,
    message: Option<(MessageType, String)>,
    tick: u32,

    store: StoreView,
    party: PartyView,
    packs: PacksView,
    games: GamesView,

    state: &'a mut State,
    conn: &'a DbConnection,
}

impl<'a> App<'a> {
    pub fn new(state: &'a mut State, conn: &'a DbConnection) -> Self {
        Self {
            route: Route::default(),
            message: None,
            state,
            tick: 0,
            store: StoreView::default(),
            party: PartyView::default(),
            packs: PacksView::default(),
            games: GamesView,
            conn,
        }
    }

    pub fn tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }

    pub fn save(&self) {
        let _ = self.state.save(self.conn);
    }

    pub fn handle(&mut self, action: Action) -> bool {
        // clear message on any action
        self.message = None;

        let result = match &mut self.route {
            Route::Store(_) => self.store.handle(action, self.state),
            Route::Party => self.party.handle(action, self.state),
            Route::Packs => self.packs.handle(action, self.state),
            Route::Games => self.games.handle(action, self.state),
        };

        match result {
            ViewResult::Redraw => {}

            ViewResult::Navigate(route) => {
                // if navigating within store, update store's sub-route
                if let Route::Store(sub_route) = route {
                    self.store.set_route(sub_route);
                }
                self.route = route;
            }

            ViewResult::Message(ty, msg) => {
                self.message = Some((ty, msg));
                self.save();
            }

            ViewResult::Exit => return false,

            ViewResult::None => {}
        }

        true
    }

    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // header + tabs
                Constraint::Min(10),   // content
                Constraint::Length(1), // footer
            ])
            .split(area);

        // header
        render_header(
            frame,
            chunks[0].inner(Margin::new(1, 0)),
            &self.route,
            self.state,
        );

        // content
        match &self.route {
            Route::Store(_) => self.store.render(frame, chunks[1], self.state, self.tick),
            Route::Party => self.party.render(frame, chunks[1], self.state, self.tick),
            Route::Packs => self.packs.render(frame, chunks[1], self.state, self.tick),
            Route::Games => self.games.render(frame, chunks[1], self.state, self.tick),
        }

        // footer
        let hints = match &self.route {
            Route::Store(_) => self.store.key_hints(),
            Route::Party => self.party.key_hints(),
            Route::Packs => self.packs.key_hints(),
            Route::Games => self.games.key_hints(),
        };
        render_footer(
            frame,
            chunks[2],
            &hints,
            self.state.party_points,
            &self.message,
        );
    }
}

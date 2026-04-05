use ratatui::prelude::*;

use crate::game::GameRef;
use crate::state::State;
use crate::storage::DbConnection;
use crate::tui::action::NUM_TABS;
use crate::tui::views::MessageType;
use crate::tui::views::pack_reveal::PackRevealView;

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
    pack_reveal: PackRevealView,
    games: GamesView,

    state: &'a mut State,
    conn: &'a DbConnection,

    /// local state for the pack reveal ceremony
    display_points_offset: u64,

    /// used to allow the TUI harness to invoke games.
    /// if this is populated, on the next iteration of the event loop,
    /// the harness will take this item and run the game
    pending_game: Option<GameRef>,
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
            pack_reveal: PackRevealView::default(),
            games: GamesView::default(),
            conn,
            display_points_offset: 0,
            pending_game: None,
        }
    }

    pub fn tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }

    pub fn save(&self) {
        let _ = self.state.save(self.conn);
    }

    pub fn reload_state(&mut self) -> anyhow::Result<()> {
        *self.state = State::load(self.conn)?;
        Ok(())
    }

    pub fn take_pending_game(&mut self) -> Option<GameRef> {
        self.pending_game.take()
    }

    pub fn set_error(&mut self, message: String) {
        self.message = Some((MessageType::Error, message))
    }

    pub fn handle(&mut self, action: Action) -> bool {
        // clear message on any action
        self.message = None;

        // handle tab navigation globally without forwarding to components
        let is_tab_nav = matches!(action, Action::Tab(_) | Action::NextTab | Action::PrevTab);

        if is_tab_nav {
            let tab_idx = match action {
                Action::Tab(n) => n,
                Action::NextTab => (self.route.tab_index() + 1) % NUM_TABS,
                Action::PrevTab => (NUM_TABS + self.route.tab_index() - 1) % NUM_TABS,
                _ => unreachable!(),
            };

            self.route = match tab_idx {
                0 => Route::Store(Default::default()),
                1 => Route::Party,
                2 => Route::Packs,
                _ => Route::Games,
            };

            return true;
        }

        // same for quit
        if matches!(action, Action::Quit) {
            return false;
        }

        let result = match &mut self.route {
            Route::Store(_) => self.store.handle(action, self.state),
            Route::Party => self.party.handle(action, self.state),
            Route::Packs => self.packs.handle(action, self.state),
            Route::PackReveal => self.pack_reveal.handle(action, self.state),
            Route::Games => self.games.handle(action, self.state),
        };

        match result {
            ViewResult::Redraw => {}

            ViewResult::Navigate(route) => {
                self.display_points_offset = 0;

                if let Route::Store(sub_route) = route {
                    self.store.set_route(sub_route);
                }
                self.route = route;
            }

            ViewResult::OpenPack(pack) => {
                let points_before = self.state.party_points;
                let pack_items = self.state.open_pack(pack);
                self.save();
                let offset = self.state.party_points - points_before;
                self.display_points_offset = offset;
                self.pack_reveal.set_items(pack_items);
                self.route = Route::PackReveal;
            }

            ViewResult::RevealPoints(points) => {
                self.display_points_offset = self.display_points_offset.saturating_sub(points);
            }

            ViewResult::Message(ty, msg) => {
                self.message = Some((ty, msg));
                self.save();
            }

            ViewResult::StartGame(game) => {
                // make sure the user can play this game
                let tokens = self.state.game_token_count(game);
                if tokens == 0 {
                    self.message = Some((
                        MessageType::Error,
                        "You don't have any tokens for this game".into(),
                    ));
                } else {
                    self.state.deduct_game_token(game);
                    self.save();
                    self.pending_game = Some(game);
                }
            }

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
            Route::PackReveal => self
                .pack_reveal
                .render(frame, chunks[1], self.state, self.tick),
            Route::Games => self.games.render(frame, chunks[1], self.state, self.tick),
        }

        // footer
        let hints = match &self.route {
            Route::Store(_) => self.store.key_hints(),
            Route::Party => self.party.key_hints(),
            Route::Packs => self.packs.key_hints(),
            Route::PackReveal => self.pack_reveal.key_hints(),
            Route::Games => self.games.key_hints(),
        };
        render_footer(
            frame,
            chunks[2],
            &hints,
            self.state.party_points - self.display_points_offset,
            &self.message,
        );
    }
}

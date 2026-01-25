use ratatui::prelude::*;

use crate::state::{self, State};

use super::action::{Action, Route};
use super::views::games::GamesView;
use super::views::packs::PacksView;
use super::views::party::PartyView;
use super::views::store::StoreView;
use super::views::{View, ViewResult};
use super::widgets::{render_footer, render_header};

pub struct App {
    pub route: Route,
    pub message: Option<String>,
    pub state: State,

    store: StoreView,
    party: PartyView,
    packs: PacksView,
    games: GamesView,
}

impl App {
    pub fn new() -> Self {
        Self {
            route: Route::default(),
            message: None,
            state: state::load(),
            store: StoreView::default(),
            party: PartyView::default(),
            packs: PacksView::default(),
            games: GamesView::default(),
        }
    }

    pub fn handle(&mut self, action: Action) -> bool {
        // clear message on any action
        self.message = None;

        let result = match &mut self.route {
            Route::Store(_) => self.store.handle(action, &mut self.state),
            Route::Party => self.party.handle(action, &mut self.state),
            Route::Packs => self.packs.handle(action, &mut self.state),
            Route::Games => self.games.handle(action, &mut self.state),
        };

        match result {
            ViewResult::None => {}
            ViewResult::Redraw => {
                let _ = state::save(&self.state);
            }
            ViewResult::Navigate(route) => {
                // if navigating within store, update store's sub-route
                if let Route::Store(sub_route) = route {
                    self.store.set_route(sub_route);
                }
                self.route = route;
            }
            ViewResult::Message(msg) => {
                self.message = Some(msg);
            }
            ViewResult::Exit => return false,
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
                Constraint::Length(1), // message or spacer
                Constraint::Length(1), // footer
            ])
            .split(area);

        // header
        render_header(frame, chunks[0], &self.route);

        // content
        match &self.route {
            Route::Store(_) => self.store.render(frame, chunks[1], &self.state),
            Route::Party => self.party.render(frame, chunks[1], &self.state),
            Route::Packs => self.packs.render(frame, chunks[1], &self.state),
            Route::Games => self.games.render(frame, chunks[1], &self.state),
        }

        // message
        if let Some(msg) = &self.message {
            let msg_widget = ratatui::widgets::Paragraph::new(msg.as_str())
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Yellow));
            frame.render_widget(msg_widget, chunks[2]);
        }

        // footer
        let hints = match &self.route {
            Route::Store(_) => self.store.key_hints(),
            Route::Party => self.party.key_hints(),
            Route::Packs => self.packs.key_hints(),
            Route::Games => self.games.key_hints(),
        };
        render_footer(frame, chunks[3], &hints, self.state.party_points);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

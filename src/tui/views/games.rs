use std::cell::Cell;

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Padding, Paragraph},
};
use tui_scrollview::{ScrollView, ScrollViewState, ScrollbarVisibility};

use crate::{
    game::{ALL_GAMES, GameRef},
    state::State,
    tui::{
        action::{Action, Route, StoreRoute},
        views::{View, ViewResult},
        widgets::ShimmerBlock,
    },
};

const ITEM_HEIGHT: u16 = 6;
const SCROLL_PADDING: u16 = ITEM_HEIGHT; // keep one item of padding when scrolling

struct GameListItem {
    game: GameRef,
    num_owned: u32,
    selected: bool,
    tick: u32,
}

impl GameListItem {
    fn new(game: GameRef, num_owned: u32, selected: bool, tick: u32) -> Self {
        Self {
            game,
            num_owned,
            selected,
            tick,
        }
    }
}

impl Widget for GameListItem {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // use shimmer block for selected, regular block for unselected
        let inner = if self.selected {
            let block = ShimmerBlock::new(self.tick);
            let inner = block.inner(area).inner(Margin::new(1, 0));
            block.render(area, buf);
            inner
        } else {
            let block = Block::default()
                .borders(Borders::ALL)
                .padding(Padding::horizontal(1))
                .border_style(Style::default().gray());
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        };

        let split = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1), // game name
            Constraint::Length(1), // count
            Constraint::Length(1),
        ])
        .split(inner);

        let name_text = Text::from(self.game.name()).reset().bold().centered();
        name_text.render(split[1], buf);

        let count_text = Text::from(format!("({})", self.num_owned))
            .dark_gray()
            .centered();
        count_text.render(split[2], buf);
    }
}

#[derive(Default)]
pub struct GamesView {
    selection: usize,

    scroll_state: ScrollViewState,
    viewport_height: Cell<u16>,
}

impl GamesView {
    fn owned_games(&self, state: &State) -> impl Iterator<Item = GameRef> {
        ALL_GAMES
            .iter()
            .filter(|&game| state.game_token_count(*game) > 0)
            .copied()
    }

    fn selected_game(&self, state: &State) -> Option<GameRef> {
        self.owned_games(state).nth(self.selection)
    }

    fn item_count(&self, state: &State) -> usize {
        self.owned_games(state).count()
    }

    fn update_scroll(&mut self) {
        let viewport_height = self.viewport_height.get();

        let selected_top = self.selection as u16 * ITEM_HEIGHT;
        let selected_bottom = selected_top + ITEM_HEIGHT;

        let current_offset = self.scroll_state.offset().y;
        let viewport_bottom = current_offset + viewport_height;

        // scroll down if selection is near bottom of viewport
        if selected_bottom + SCROLL_PADDING > viewport_bottom {
            let new_offset = (selected_bottom + SCROLL_PADDING).saturating_sub(viewport_height);
            self.scroll_state.set_offset(Position::new(0, new_offset));
        }
        // scroll up if selection is near top of viewport
        else if selected_top < current_offset + SCROLL_PADDING {
            let new_offset = selected_top.saturating_sub(SCROLL_PADDING);
            self.scroll_state.set_offset(Position::new(0, new_offset));
        }
    }
}

impl View for GamesView {
    fn render(&self, frame: &mut Frame, area: Rect, state: &State, tick: u32) {
        self.viewport_height.set(area.height);

        // split out header
        let split = Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).split(area);

        let block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().dark_gray());
        let header = Paragraph::new("Game Collection")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Reset))
            .block(block);
        frame.render_widget(header, split[0]);

        // just show a centered message if there are no game tokens
        if self.item_count(state) == 0 {
            let split = Layout::vertical([
                Constraint::Fill(2),
                Constraint::Length(1), // message here
                Constraint::Fill(3),
            ])
            .split(split[1]);

            let text =
                Text::from("You don't have any game tokens. Go buy some or open some packs!")
                    .dark_gray()
                    .centered();
            frame.render_widget(text, split[2]);
            return;
        }

        // if there are game tokens, list them
        let content_area = split[1].inner(Margin::new(1, 0));
        let content_width = content_area.width.saturating_sub(1); // leave room for scrollbar
        let content_height = self.item_count(state) as u16 * ITEM_HEIGHT;

        let mut scroll_view = ScrollView::new(Size::new(content_width, content_height))
            .horizontal_scrollbar_visibility(ScrollbarVisibility::Never);

        // render items into scroll view
        let owned_games = self.owned_games(state);
        for (i, game) in owned_games.enumerate() {
            let selected = self.selection == i;
            let num_owned = state.game_token_count(game);

            let item = GameListItem::new(game, num_owned, selected, tick);
            let item_rect = Rect::new(0, i as u16 * ITEM_HEIGHT, content_width, ITEM_HEIGHT);
            scroll_view.render_widget(item, item_rect);
        }

        // render scroll view
        frame.render_stateful_widget(scroll_view, content_area, &mut self.scroll_state.clone());
    }

    fn handle(&mut self, action: Action, state: &mut State) -> ViewResult {
        match action {
            Action::Up => {
                let count = self.item_count(state);
                self.selection = (self.selection + count - 1) % count;
                self.update_scroll(); // approximate viewport height
                ViewResult::Redraw
            }
            Action::Down => {
                self.selection = (self.selection + 1) % self.item_count(state);
                self.update_scroll();
                ViewResult::Redraw
            }
            Action::Select => {
                if let Some(game) = self.selected_game(state) {
                    ViewResult::StartGame(game)
                } else {
                    ViewResult::None
                }
            }
            Action::Back => ViewResult::Navigate(Route::Store(StoreRoute::Grid)),
            _ => ViewResult::None,
        }
    }

    fn key_hints(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("↑↓", "select"),
            ("enter", "open"),
            ("esc", "back"),
            ("q", "quit"),
        ]
    }
}

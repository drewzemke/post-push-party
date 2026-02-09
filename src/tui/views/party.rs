use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Padding};
use tui_scrollview::{ScrollView, ScrollViewState, ScrollbarVisibility};

use crate::party::{Party, ALL_PARTIES};
use crate::state::State;
use crate::tui::widgets::ShimmerBlock;

use super::{Action, Route, View, ViewResult};

const ITEM_HEIGHT: u16 = 5;
const SCROLL_PADDING: u16 = ITEM_HEIGHT;

struct PartyItem {
    party: &'static dyn Party,
    enabled: bool,
    selected: bool,
    tick: u32,
}

impl PartyItem {
    fn new(party: &'static dyn Party, enabled: bool, selected: bool, tick: u32) -> Self {
        Self {
            party,
            enabled,
            selected,
            tick,
        }
    }
}

impl Widget for PartyItem {
    fn render(self, area: Rect, buf: &mut Buffer) {
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

        let chunks = Layout::vertical([
            Constraint::Length(1), // name
            Constraint::Length(1), // description
            Constraint::Length(1), // status
        ])
        .split(inner);

        // name
        let title = Text::from(self.party.name()).reset().bold();
        title.render(chunks[0], buf);

        // description
        let desc = Text::from(self.party.description()).dark_gray();
        desc.render(chunks[1], buf);

        // enabled status
        let (status_text, status_style) = if self.enabled {
            ("✓ Enabled", Style::default().fg(Color::Green))
        } else {
            ("✗ Disabled", Style::default().fg(Color::Red))
        };

        let status = Text::from(status_text).style(status_style);
        status.render(chunks[2], buf);
    }
}

#[derive(Default)]
pub struct PartyView {
    selection: usize,
    scroll_state: ScrollViewState,
}

impl PartyView {
    fn unlocked_parties(state: &State) -> impl Iterator<Item = &'static dyn Party> + use<'_> {
        ALL_PARTIES
            .iter()
            .map(|p| *p)
            .filter(|&party| state.is_party_unlocked(party.id()))
    }

    fn item_count(state: &State) -> usize {
        Self::unlocked_parties(state).count()
    }

    fn selected_party(&self, state: &State) -> Option<&'static dyn Party> {
        Self::unlocked_parties(state).nth(self.selection)
    }

    fn update_scroll(&mut self, viewport_height: u16) {
        let selected_top = self.selection as u16 * ITEM_HEIGHT;
        let selected_bottom = selected_top + ITEM_HEIGHT;

        let current_offset = self.scroll_state.offset().y;
        let viewport_bottom = current_offset + viewport_height;

        if selected_bottom + SCROLL_PADDING > viewport_bottom {
            let new_offset = (selected_bottom + SCROLL_PADDING).saturating_sub(viewport_height);
            self.scroll_state.set_offset(Position::new(0, new_offset));
        } else if selected_top < current_offset + SCROLL_PADDING {
            let new_offset = selected_top.saturating_sub(SCROLL_PADDING);
            self.scroll_state.set_offset(Position::new(0, new_offset));
        }
    }
}

impl View for PartyView {
    fn render(&self, frame: &mut Frame, area: Rect, state: &State, tick: u32) {
        let content_area = area.inner(Margin::new(1, 0));
        let content_width = content_area.width.saturating_sub(1); // leave room for scrollbar
        let content_height = Self::item_count(state) as u16 * ITEM_HEIGHT;

        let mut scroll_view = ScrollView::new(Size::new(content_width, content_height))
            .horizontal_scrollbar_visibility(ScrollbarVisibility::Never);

        for (i, party) in Self::unlocked_parties(state).enumerate() {
            let enabled = state.is_party_enabled(party.id());
            let selected = self.selection == i;

            let item = PartyItem::new(party, enabled, selected, tick);
            let item_rect = Rect::new(0, i as u16 * ITEM_HEIGHT, content_width, ITEM_HEIGHT);
            scroll_view.render_widget(item, item_rect);
        }

        frame.render_stateful_widget(scroll_view, content_area, &mut self.scroll_state.clone());
    }

    fn handle(&mut self, action: Action, state: &mut State) -> ViewResult {
        match action {
            Action::Up => {
                let count = Self::item_count(state);
                self.selection = (self.selection + count - 1) % count;
                self.update_scroll(20);
                ViewResult::Redraw
            }
            Action::Down => {
                self.selection = (self.selection + 1) % Self::item_count(state);
                self.update_scroll(20);
                ViewResult::Redraw
            }
            Action::Select => {
                if let Some(party) = self.selected_party(state) {
                    state.toggle_party(party.id());
                }
                ViewResult::Redraw
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
            ("↑↓", "select"),
            ("Enter", "toggle"),
            ("1-4", "tab"),
            ("q", "quit"),
        ]
    }
}

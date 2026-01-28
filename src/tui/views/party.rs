use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Padding};
use tui_scrollview::{ScrollView, ScrollViewState, ScrollbarVisibility};

use crate::state::{PartyFeature, State, PARTY_FEATURES};
use crate::tui::views::MessageType;
use crate::tui::widgets::ShimmerBlock;

use super::{Action, Route, View, ViewResult};

const ITEM_HEIGHT: u16 = 5;
const SCROLL_PADDING: u16 = ITEM_HEIGHT;

struct PartyItem {
    name: &'static str,
    description: &'static str,
    status: ItemStatus,
    selected: bool,
    tick: u32,
}

enum ItemStatus {
    Enabled,
    Disabled,
}

impl PartyItem {
    fn new(name: &'static str, description: &'static str, status: ItemStatus, selected: bool, tick: u32) -> Self {
        Self { name, description, status, selected, tick }
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
        let title = Text::from(self.name).reset().bold();
        title.render(chunks[0], buf);

        // description
        let desc = Text::from(self.description).dark_gray();
        desc.render(chunks[1], buf);

        // status
        let (status_text, status_style) = match self.status {
            ItemStatus::Enabled => ("✓ Enabled", Style::default().fg(Color::Green)),
            ItemStatus::Disabled => ("✗ Disabled", Style::default().fg(Color::Red)),
        };
        let status = Text::from(status_text).style(status_style);
        status.render(chunks[2], buf);
    }
}

pub struct PartyView {
    selection: usize,
    scroll_state: ScrollViewState,
}

impl Default for PartyView {
    fn default() -> Self {
        Self {
            selection: 0,
            scroll_state: ScrollViewState::default(),
        }
    }
}

impl PartyView {
    fn unlocked_features(state: &State) -> Vec<PartyFeature> {
        PARTY_FEATURES
            .iter()
            .copied()
            .filter(|&f| state.is_unlocked(f))
            .collect()
    }

    fn item_count(state: &State) -> usize {
        1 + Self::unlocked_features(state).len() // basic party + unlocked features
    }

    fn selected_feature(&self, state: &State) -> Option<PartyFeature> {
        if self.selection == 0 {
            None
        } else {
            Self::unlocked_features(state).get(self.selection - 1).copied()
        }
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
        let unlocked = Self::unlocked_features(state);

        let content_area = area.inner(Margin::new(1, 0));
        let content_width = content_area.width;
        let content_height = Self::item_count(state) as u16 * ITEM_HEIGHT;

        let mut scroll_view = ScrollView::new(Size::new(content_width, content_height))
            .horizontal_scrollbar_visibility(ScrollbarVisibility::Never);

        // basic party (always enabled)
        let basic_item = PartyItem::new(
            "Basic Party",
            "A simple summary of how many points you earned.",
            ItemStatus::Enabled,
            self.selection == 0,
            tick,
        );
        scroll_view.render_widget(basic_item, Rect::new(0, 0, content_width, ITEM_HEIGHT));

        // unlocked features only
        for (i, feature) in unlocked.iter().enumerate() {
            let status = if state.is_enabled(*feature) {
                ItemStatus::Enabled
            } else {
                ItemStatus::Disabled
            };

            let item = PartyItem::new(feature.name(), feature.description(), status, self.selection == i + 1, tick);
            let item_rect = Rect::new(0, (i + 1) as u16 * ITEM_HEIGHT, content_width, ITEM_HEIGHT);
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
                if let Some(feature) = self.selected_feature(state) {
                    state.toggle_feature(feature);
                    ViewResult::Redraw
                } else {
                    ViewResult::Message(
                        MessageType::Normal,
                        "Basic party is always enabled".to_string(),
                    )
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
            ("↑↓", "select"),
            ("Enter", "toggle"),
            ("1-4", "tab"),
            ("q", "quit"),
        ]
    }
}

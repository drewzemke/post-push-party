use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};
use tui_scrollview::{ScrollView, ScrollViewState, ScrollbarVisibility};

use crate::party::{Party, ALL_PARTIES};
use crate::state::State;
use crate::tui::action::{Action, Route, StoreRoute};
use crate::tui::views::{MessageType, View, ViewResult};
use crate::tui::widgets::ShimmerBlock;

const ITEM_HEIGHT: u16 = 4;
const SCROLL_PADDING: u16 = ITEM_HEIGHT; // keep one item of padding when scrolling

struct PartyListItem {
    party: &'static dyn Party,
    unlocked: bool,
    affordable: bool,
    selected: bool,
    tick: u32,
}

impl PartyListItem {
    fn new(
        party: &'static dyn Party,
        unlocked: bool,
        affordable: bool,
        selected: bool,
        tick: u32,
    ) -> Self {
        Self {
            party,
            unlocked,
            affordable,
            selected,
            tick,
        }
    }
}

impl Widget for PartyListItem {
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

        // divide inner into top and bottom rows
        let chunks = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(inner);

        // top line -- split into title and price
        let top_chunks =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).split(chunks[0]);

        let title_text = Text::from(self.party.name()).reset().bold();
        title_text.render(top_chunks[0], buf);

        let price_style = if self.unlocked {
            Style::default().fg(Color::DarkGray)
        } else if self.affordable {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Red)
        };

        let price_str = if self.unlocked {
            "✓ Owned".to_string()
        } else {
            format!("{} P", self.party.cost())
        };

        let price_text = Text::from(price_str)
            .style(price_style)
            .alignment(Alignment::Right);
        price_text.render(top_chunks[1], buf);

        // bottom line -- just the description
        let desc_text = Text::from(self.party.description()).reset();
        desc_text.render(chunks[1], buf);
    }
}

#[derive(Default)]
pub struct UpgradesView {
    selection: usize,
    scroll_state: ScrollViewState,
}

impl UpgradesView {
    fn selected_party(&self) -> Option<&'static dyn Party> {
        ALL_PARTIES.get(self.selection).copied()
    }

    const fn item_count(&self) -> usize {
        ALL_PARTIES.len()
    }

    fn update_scroll(&mut self, viewport_height: u16) {
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

impl View for UpgradesView {
    fn render(&self, frame: &mut Frame, area: Rect, state: &State, tick: u32) {
        // split out header
        let chunks = Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).split(area);

        let block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().dark_gray());
        let header = Paragraph::new("Party Upgrades")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Reset))
            .block(block);
        frame.render_widget(header, chunks[0]);

        // content area
        let content_area = chunks[1].inner(Margin::new(1, 0));
        let content_width = content_area.width.saturating_sub(1); // leave room for scrollbar
        let content_height = self.item_count() as u16 * ITEM_HEIGHT;

        let mut scroll_view = ScrollView::new(Size::new(content_width, content_height))
            .horizontal_scrollbar_visibility(ScrollbarVisibility::Never);

        // render items into scroll view
        for (i, &party) in ALL_PARTIES.iter().enumerate() {
            let affordable = state.party_points >= party.cost();
            let selected = self.selection == i;
            let unlocked = state.is_party_unlocked(party.id());

            let item = PartyListItem::new(party, unlocked, affordable, selected, tick);
            let item_rect = Rect::new(0, i as u16 * ITEM_HEIGHT, content_width, ITEM_HEIGHT);
            scroll_view.render_widget(item, item_rect);
        }

        // render scroll view
        frame.render_stateful_widget(scroll_view, content_area, &mut self.scroll_state.clone());
    }

    fn handle(&mut self, action: Action, state: &mut State) -> ViewResult {
        match action {
            Action::Up => {
                let count = self.item_count();
                self.selection = (self.selection + count - 1) % count;
                self.update_scroll(20); // approximate viewport height
                ViewResult::Redraw
            }
            Action::Down => {
                self.selection = (self.selection + 1) % self.item_count();
                self.update_scroll(20);
                ViewResult::Redraw
            }
            Action::Select => {
                if let Some(party) = self.selected_party() {
                    if state.is_party_unlocked(party.id()) {
                        ViewResult::Message(
                            MessageType::Normal,
                            format!("You already own {}.", party.name()),
                        )
                    } else {
                        let cost = party.cost();
                        if state.party_points >= cost {
                            state.party_points -= cost;
                            state.unlock_party(party.id());
                            ViewResult::Message(
                                MessageType::Success,
                                format!("Unlocked {}!", party.name()),
                            )
                        } else {
                            ViewResult::Message(
                                MessageType::Error,
                                format!("You need {} more points.", cost - state.party_points),
                            )
                        }
                    }
                } else {
                    ViewResult::None
                }
            }
            Action::Back => ViewResult::Navigate(Route::Store(StoreRoute::Grid)),
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
            ("Enter", "buy"),
            ("Esc", "back"),
            ("q", "quit"),
        ]
    }
}

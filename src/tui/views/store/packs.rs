use std::cell::Cell;

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
};
use tui_scrollview::{ScrollView, ScrollViewState, ScrollbarVisibility};

use crate::{
    pack::{ALL_PACKS, Pack},
    state::State,
    tui::{
        action::{Action, Route, StoreRoute},
        views::{MessageType, View, ViewResult},
        widgets::ShimmerBlock,
    },
};

const ITEM_HEIGHT: u16 = 5;
const SCROLL_PADDING: u16 = ITEM_HEIGHT; // keep one item of padding when scrolling
const OWNED_COUNT_WIDTH: u16 = 17;

struct PackListItem {
    pack: Pack,
    num_owned: u32,
    affordable: bool,
    selected: bool,
    tick: u32,
}

impl PackListItem {
    fn new(pack: Pack, num_owned: u32, affordable: bool, selected: bool, tick: u32) -> Self {
        Self {
            pack,
            num_owned,
            affordable,
            selected,
            tick,
        }
    }
}

impl Widget for PackListItem {
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
        let main_split =
            Layout::vertical([Constraint::Length(1), Constraint::Length(2)]).split(inner);

        // top line -- split into title and price
        let top_split =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).split(main_split[0]);

        let title_text = Text::from(self.pack.name()).reset().bold();
        title_text.render(top_split[0], buf);

        let price_style = if self.affordable {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Red)
        };

        let price_str = format!("{} P", self.pack.cost());

        let price_text = Text::from(price_str)
            .style(price_style)
            .alignment(Alignment::Right);
        price_text.render(top_split[1], buf);

        // bottom line -- split into description and owned count
        let bottom_split =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(OWNED_COUNT_WIDTH)])
                .split(main_split[1]);

        let desc_text = Paragraph::new(self.pack.description())
            .reset()
            .wrap(Wrap::default());
        desc_text.render(bottom_split[0], buf);

        if self.num_owned > 0 {
            let num_owned_str = format!("In Inventory: {}", self.num_owned);
            let price_text = Text::from(num_owned_str)
                .dark_gray()
                .alignment(Alignment::Right);
            // render on the bottom line
            let split = Layout::vertical([Constraint::Length(1), Constraint::Length(1)])
                .split(bottom_split[1]);
            price_text.render(split[1], buf);
        }
    }
}

#[derive(Default)]
pub struct PacksView {
    selection: usize,

    scroll_state: ScrollViewState,
    viewport_height: Cell<u16>,
}

impl PacksView {
    fn selected_pack(&self) -> Option<Pack> {
        ALL_PACKS.get(self.selection).copied()
    }

    const fn item_count(&self) -> usize {
        ALL_PACKS.len()
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

impl View for PacksView {
    fn render(&self, frame: &mut Frame, area: Rect, state: &State, tick: u32) {
        self.viewport_height.set(area.height);

        // split out header
        let split = Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).split(area);

        let block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().dark_gray());
        let header = Paragraph::new("Packs")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Reset))
            .block(block);
        frame.render_widget(header, split[0]);

        // content area
        let content_area = split[1].inner(Margin::new(1, 0));
        let content_width = content_area.width.saturating_sub(1); // leave room for scrollbar
        let content_height = self.item_count() as u16 * ITEM_HEIGHT;

        let mut scroll_view = ScrollView::new(Size::new(content_width, content_height))
            .horizontal_scrollbar_visibility(ScrollbarVisibility::Never);

        // render items into scroll view
        for (i, &pack) in ALL_PACKS.iter().enumerate() {
            let affordable = state.party_points >= pack.cost();
            let selected = self.selection == i;
            let num_owned = state.pack_count(pack);

            let item = PackListItem::new(pack, num_owned, affordable, selected, tick);
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
                self.update_scroll(); // approximate viewport height
                ViewResult::Redraw
            }
            Action::Down => {
                self.selection = (self.selection + 1) % self.item_count();
                self.update_scroll();
                ViewResult::Redraw
            }
            Action::Select => {
                if let Some(pack) = self.selected_pack() {
                    let cost = pack.cost();
                    if state.party_points >= cost {
                        state.party_points -= cost;
                        state.add_pack(pack);
                        ViewResult::Message(
                            MessageType::Success,
                            format!("Purchased a {}!", pack.name()),
                        )
                    } else {
                        ViewResult::Message(
                            MessageType::Error,
                            format!("You need {} more points.", cost - state.party_points),
                        )
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
            ("enter", "buy"),
            ("esc", "back"),
            ("q", "quit"),
        ]
    }
}

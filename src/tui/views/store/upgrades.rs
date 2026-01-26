use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use tui_scrollview::{ScrollView, ScrollViewState, ScrollbarVisibility};

use crate::state::{feature_cost, PartyFeature, State, PARTY_FEATURES};
use crate::tui::action::{Action, Route, StoreRoute};
use crate::tui::views::{MessageType, View, ViewResult};
use crate::tui::widgets::Card;

const ITEM_HEIGHT: u16 = 4;
const SCROLL_PADDING: u16 = ITEM_HEIGHT; // keep one item of padding when scrolling

struct UpgradeItem {
    feature: PartyFeature,
    unlocked: bool,
    affordable: bool,
    cost: u64,
    selected: bool,
}

impl UpgradeItem {
    fn new(feature: PartyFeature, unlocked: bool, affordable: bool, cost: u64, selected: bool) -> Self {
        Self { feature, unlocked, affordable, cost, selected }
    }

    fn description(&self) -> &'static str {
        match self.feature {
            PartyFeature::Exclamations => "Adds an excited shout to your party.",
            PartyFeature::Quotes => "An inspirational quote after each push.",
            PartyFeature::BigText => "Finish your party with a full screen word. NICE!",
        }
    }
}

impl Widget for UpgradeItem {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let price_style = if self.unlocked {
            Style::default().fg(Color::DarkGray)
        } else if self.affordable {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Red)
        };

        let price_text = if self.unlocked {
            "✓ Owned".to_string()
        } else {
            format!("{} P", self.cost)
        };

        // title line with price right-aligned
        let title_width = self.feature.name().len();
        let price_width = price_text.len();
        let card_inner_width = area.width.saturating_sub(4) as usize; // borders
        let spacing = card_inner_width.saturating_sub(title_width + price_width);

        let title_line = Line::from(vec![
            Span::styled(self.feature.name(), Style::default().fg(Color::White)),
            Span::raw(" ".repeat(spacing)),
            Span::styled(price_text, price_style),
        ]);

        let card = Card::new()
            .content(vec![title_line, Line::from(self.description())])
            .selected(self.selected);

        card.render(area, buf);
    }
}

pub struct UpgradesView {
    selection: usize,
    scroll_state: ScrollViewState,
}

impl Default for UpgradesView {
    fn default() -> Self {
        Self {
            selection: 0,
            scroll_state: ScrollViewState::default(),
        }
    }
}

impl UpgradesView {
    fn selected_feature(&self) -> Option<PartyFeature> {
        PARTY_FEATURES.get(self.selection).copied()
    }

    fn item_count(&self) -> usize {
        PARTY_FEATURES.len()
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
    fn render(&self, frame: &mut Frame, area: Rect, state: &State) {
        // split out header
        let chunks = Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).split(area);

        let block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().dark_gray());
        let header = Paragraph::new("Upgrades")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White))
            .block(block);
        frame.render_widget(header, chunks[0]);

        // content area
        let content_area = chunks[1].inner(Margin::new(1, 0));
        let content_width = content_area.width;
        let content_height = self.item_count() as u16 * ITEM_HEIGHT;

        let mut scroll_view = ScrollView::new(Size::new(content_width, content_height))
            .horizontal_scrollbar_visibility(ScrollbarVisibility::Never);

        // render items into scroll view
        for (i, &feature) in PARTY_FEATURES.iter().enumerate() {
            let item = UpgradeItem::new(
                feature,
                state.is_unlocked(feature),
                state.party_points >= feature_cost(feature),
                feature_cost(feature),
                self.selection == i,
            );
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
                if let Some(feature) = self.selected_feature() {
                    if state.is_unlocked(feature) {
                        ViewResult::Message(
                            MessageType::Normal,
                            format!("{} already owned", feature.name()),
                        )
                    } else {
                        let cost = feature_cost(feature);
                        if state.party_points >= cost {
                            state.party_points -= cost;
                            state.unlock_feature(feature);
                            ViewResult::Message(
                                MessageType::Success,
                                format!("Unlocked {}!", feature.name()),
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

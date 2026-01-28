use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};
use tui_scrollview::{ScrollView, ScrollViewState, ScrollbarVisibility};

use crate::state::State;
use crate::tui::action::{Action, Route, StoreRoute};
use crate::tui::views::{MessageType, View, ViewResult};
use crate::tui::widgets::ShimmerBlock;

const ITEM_HEIGHT: u16 = 8;
const SCROLL_PADDING: u16 = ITEM_HEIGHT;
const TIER_WIDTH: u16 = 10;

#[derive(Clone)]
struct BonusTrack {
    name: &'static str,
    description: &'static str,
    tiers: &'static [(&'static str, u64)],
}

const BONUS_TRACKS: &[BonusTrack] = &[
    BonusTrack {
        name: "Commit Value",
        description: "How many party points you get per commit.",
        tiers: &[("1", 0), ("2", 50), ("3", 500), ("4", 5000), ("5", 50000)],
    },
    BonusTrack {
        name: "Weekend Warrior",
        description: "Earn more points for pushing code on Saturday or Sunday.",
        tiers: &[
            ("1x", 100),
            ("2x", 500),
            ("3x", 1000),
            ("4x", 5000),
            ("5x", 10000),
        ],
    },
    BonusTrack {
        name: "First Catch of the Day",
        description: "Make your first push each day more valuable.",
        tiers: &[
            ("1x", 100),
            ("2x", 500),
            ("3x", 1000),
            ("4x", 5000),
            ("5x", 10000),
            ("6x", 50000),
            ("7x", 100000),
            ("8x", 500000),
            ("9x", 1000000),
            ("10x", 5000000),
        ],
    },
];

struct BonusItem<'a> {
    track: BonusTrack,
    state: &'a State,
    selected: bool,
    tick: u32,
    owned_level: usize, // FIXME: temporary fixture data
}

impl<'a> BonusItem<'a> {
    fn new(
        track: BonusTrack,
        state: &'a State,
        selected: bool,
        tick: u32,
        owned_level: usize,
    ) -> Self {
        Self {
            track,
            state,
            selected,
            tick,
            owned_level,
        }
    }
}

impl<'a> Widget for BonusItem<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let owned_level = self.owned_level;

        // shimmer border for selected row, plain for unselected
        let inner = if self.selected {
            let block = ShimmerBlock::new(self.tick);
            let inner = block.inner(area).inner(Margin::new(1, 0));
            block.render(area, buf);
            inner
        } else {
            let block = Block::default()
                .border_style(Style::default().gray())
                .padding(Padding::horizontal(1))
                .borders(Borders::ALL);
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        };

        let chunks = Layout::vertical([
            Constraint::Length(1), // title
            Constraint::Length(1), // description
            Constraint::Length(4), // tiers
        ])
        .split(inner);

        // title
        let title = Text::from(self.track.name).reset().bold();
        title.render(chunks[0], buf);

        // description
        let description = Text::from(self.track.description).reset();
        description.render(chunks[1], buf);

        // tiers: show current tier first, then next (gold), then rest
        let tiers_area = chunks[2];
        let max_visible = (tiers_area.width / TIER_WIDTH) as usize;

        // start from current tier (last owned), or 0 if none owned
        let start_idx = owned_level.saturating_sub(1);
        let end_idx = (start_idx + max_visible).min(self.track.tiers.len());
        let visible_count = end_idx - start_idx;

        let tiers_constraints: Vec<_> = (0..visible_count)
            .map(|_| Constraint::Length(TIER_WIDTH))
            .collect();
        let tiers_chunks = Layout::horizontal(tiers_constraints).split(tiers_area);

        for (render_idx, idx) in (start_idx..end_idx).enumerate() {
            let (tier_label, tier_cost) = &self.track.tiers[idx];
            let is_current = idx == owned_level.saturating_sub(1) && owned_level > 0;
            let is_next = idx == owned_level && self.selected;
            let affordable = self.state.party_points >= *tier_cost;

            // text style: white for current, green/red for next based on affordability
            let style = if is_current {
                Style::default().fg(Color::White).bold()
            } else if is_next {
                let color = if affordable { Color::Green } else { Color::Red };
                Style::default().fg(color).bold()
            } else {
                Style::default().fg(Color::DarkGray)
            };

            // border style: white for current, green/red for next, dark gray otherwise
            let border_style = if is_current {
                Style::default().fg(Color::White)
            } else if is_next {
                let color = if affordable { Color::Green } else { Color::Red };
                Style::default().fg(color)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let chunk = tiers_chunks[render_idx];

            let block = Block::default()
                .border_style(border_style)
                .borders(Borders::ALL);
            let inner = block.inner(chunk);
            block.render(chunk, buf);

            let inner_chunks =
                Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(inner);

            let label_text = Text::from(*tier_label)
                .style(style)
                .alignment(Alignment::Center);
            label_text.render(inner_chunks[0], buf);

            let cost_text = Text::from(format_cost(*tier_cost))
                .style(style)
                .alignment(Alignment::Center);
            cost_text.render(inner_chunks[1], buf);
        }
    }
}

pub struct BonusesView {
    selection: usize,
    scroll_state: ScrollViewState,
}

impl Default for BonusesView {
    fn default() -> Self {
        Self {
            selection: 0,
            scroll_state: ScrollViewState::default(),
        }
    }
}

impl BonusesView {
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

impl View for BonusesView {
    fn render(&self, frame: &mut Frame, area: Rect, state: &State, tick: u32) {
        let chunks = Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).split(area);

        // sub-header
        let block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().dark_gray());
        let header = Paragraph::new("Bonuses")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Reset))
            .block(block);
        frame.render_widget(header, chunks[0]);

        // content area with scrollview
        let content_area = chunks[1].inner(Margin::new(1, 0));
        let content_width = content_area.width;
        let content_height = BONUS_TRACKS.len() as u16 * ITEM_HEIGHT;

        let mut scroll_view = ScrollView::new(Size::new(content_width, content_height))
            .horizontal_scrollbar_visibility(ScrollbarVisibility::Never);

        // FIXME: fixture owned levels for testing
        let owned_levels = [1, 1, 8];

        for (i, track) in BONUS_TRACKS.iter().enumerate() {
            let owned_level = owned_levels.get(i).copied().unwrap_or(0);
            let item = BonusItem::new(track.clone(), state, self.selection == i, tick, owned_level);
            let item_rect = Rect::new(0, i as u16 * ITEM_HEIGHT, content_width, ITEM_HEIGHT);
            scroll_view.render_widget(item, item_rect);
        }

        frame.render_stateful_widget(scroll_view, content_area, &mut self.scroll_state.clone());
    }

    fn handle(&mut self, action: Action, _state: &mut State) -> ViewResult {
        match action {
            Action::Up => {
                let count = BONUS_TRACKS.len();
                self.selection = (self.selection + count - 1) % count;
                self.update_scroll(20);
                ViewResult::Redraw
            }
            Action::Down => {
                self.selection = (self.selection + 1) % BONUS_TRACKS.len();
                self.update_scroll(20);
                ViewResult::Redraw
            }
            Action::Select => {
                // TODO: implement purchasing
                ViewResult::Message(
                    MessageType::Normal,
                    "Bonus purchasing coming soon...".to_string(),
                )
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

fn format_cost(cost: u64) -> String {
    if cost >= 1_000_000 {
        format!("{}M P", cost / 1_000_000)
    } else if cost >= 1_000 {
        format!("{}K P", cost / 1_000)
    } else {
        format!("{} P", cost)
    }
}

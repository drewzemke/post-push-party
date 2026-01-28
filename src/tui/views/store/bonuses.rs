use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};
use tui_scrollview::{ScrollView, ScrollViewState, ScrollbarVisibility};

use crate::state::State;

const ITEM_HEIGHT: u16 = 8;
const SCROLL_PADDING: u16 = ITEM_HEIGHT;
const TIER_WIDTH: u16 = 10;
use crate::tui::action::{Action, Route, StoreRoute};
use crate::tui::views::{MessageType, View, ViewResult};

#[derive(Clone)]
struct BonusTrack {
    name: &'static str,
    description: &'static str,
    tiers: &'static [(&'static str, u64)], // (label, cost) - cost 0 means owned
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
    tier_selection: usize,
    tier_scroll_offset: usize,
}

impl<'a> BonusItem<'a> {
    fn new(
        track: BonusTrack,
        state: &'a State,
        selected: bool,
        tier_selection: usize,
        tier_scroll_offset: usize,
    ) -> Self {
        Self {
            track,
            state,
            selected,
            tier_selection,
            tier_scroll_offset,
        }
    }
}

impl<'a> Widget for BonusItem<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // FIXME using commit value level for eveything
        let owned_level = self.state.commit_value_level as usize;

        // colored border based on selection
        let border_style = if self.selected {
            Style::default().cyan()
        } else {
            Style::reset()
        };

        let block = Block::default()
            .border_style(border_style)
            .padding(Padding::horizontal(1))
            .borders(Borders::ALL);

        let inner = block.inner(area);

        block.render(area, buf);

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

        // tiers - calculate visible range based on scroll offset
        let tiers_area = chunks[2];
        let visible_count = (tiers_area.width / TIER_WIDTH) as usize;
        let end_idx = (self.tier_scroll_offset + visible_count).min(self.track.tiers.len());

        let visible_tiers: Vec<_> = self.track.tiers[self.tier_scroll_offset..end_idx]
            .iter()
            .enumerate()
            .collect();

        let tiers_constraints: Vec<_> = visible_tiers
            .iter()
            .map(|_| Constraint::Length(TIER_WIDTH))
            .collect();
        let tiers_chunks = Layout::horizontal(tiers_constraints).split(tiers_area);

        for (render_idx, (local_idx, (tier_label, tier_cost))) in
            visible_tiers.into_iter().enumerate()
        {
            let actual_idx = self.tier_scroll_offset + local_idx;
            let is_owned = actual_idx < owned_level;
            let is_tier_selected = self.selected && actual_idx == self.tier_selection;

            let style = if is_tier_selected {
                Style::default().fg(Color::Cyan).bold()
            } else if is_owned {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let border_style = if is_tier_selected {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let chunk = tiers_chunks[render_idx];

            let block = Block::default()
                .border_style(border_style)
                .borders(Borders::ALL);
            let inner = block.inner(chunk);

            block.render(chunk, buf);

            // split inner into top and bottom
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
    tier_selections: Vec<usize>,
    tier_scroll_offsets: Vec<usize>,
    scroll_state: ScrollViewState,
}

impl Default for BonusesView {
    fn default() -> Self {
        Self {
            selection: 0,
            tier_selections: vec![2; BONUS_TRACKS.len()],
            tier_scroll_offsets: vec![0; BONUS_TRACKS.len()],
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

    fn update_tier_scroll(&mut self, visible_count: usize) {
        let tier_selection = self.tier_selections[self.selection];
        let tier_scroll_offset = &mut self.tier_scroll_offsets[self.selection];

        // scroll right if selection is past visible range
        if tier_selection >= *tier_scroll_offset + visible_count {
            *tier_scroll_offset = tier_selection + 1 - visible_count;
        }
        // scroll left if selection is before visible range
        else if tier_selection < *tier_scroll_offset {
            *tier_scroll_offset = tier_selection;
        }
    }
}

impl View for BonusesView {
    fn render(&self, frame: &mut Frame, area: Rect, state: &State, _tick: u32) {
        // split out header
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

        // bonus tracks
        for (i, track) in BONUS_TRACKS.iter().enumerate() {
            let selected = self.selection == i;

            let item = BonusItem::new(
                track.clone(),
                state,
                selected,
                self.tier_selections[i],
                self.tier_scroll_offsets[i],
            );
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
            Action::Left => {
                if self.tier_selections[self.selection] > 0 {
                    self.tier_selections[self.selection] -= 1;
                    self.update_tier_scroll(5); // approximate visible count
                }
                ViewResult::Redraw
            }
            Action::Right => {
                let tier_count = BONUS_TRACKS[self.selection].tiers.len();
                if self.tier_selections[self.selection] < tier_count - 1 {
                    self.tier_selections[self.selection] += 1;
                    self.update_tier_scroll(5); // approximate visible count
                }
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
        }
    }

    fn key_hints(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("↑↓", "track"),
            ("←→", "tier"),
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

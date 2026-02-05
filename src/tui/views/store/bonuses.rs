use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use tui_scrollview::{ScrollView, ScrollViewState, ScrollbarVisibility};

use crate::bonus_tracks::{Reward, Tier, ALL_TRACKS};
use crate::state::State;
use crate::tui::action::{Action, Route, StoreRoute};
use crate::tui::views::{MessageType, View, ViewResult};
use crate::tui::widgets::ShimmerBlock;

const ITEM_HEIGHT: u16 = 5;
const SCROLL_PADDING: u16 = ITEM_HEIGHT;
const COST_INFO_WIDTH: u16 = 30;

struct BonusItem<'a> {
    name: &'static str,
    description: &'static str,
    tiers: &'static [Tier],
    state: &'a State,
    selected: bool,
    tick: u32,
    owned_level: usize,
}

impl<'a> BonusItem<'a> {
    fn new(
        name: &'static str,
        description: &'static str,
        tiers: &'static [Tier],
        state: &'a State,
        selected: bool,
        tick: u32,
        owned_level: usize,
    ) -> Self {
        Self {
            name,
            description,
            tiers,
            state,
            selected,
            tick,
            owned_level,
        }
    }
}

impl<'a> Widget for BonusItem<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // shimmer border for selected row, plain for unselected
        let inner = if self.selected {
            let block = ShimmerBlock::new(self.tick);
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        } else {
            let block = Block::default()
                .border_style(Style::default().dim())
                .borders(Borders::ALL);
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        };

        // three horizontal segments
        let chunks = Layout::horizontal([
            Constraint::Fill(1),                 // name + description
            Constraint::Length(COST_INFO_WIDTH), // cost info
        ])
        .split(inner);

        // name + description
        let name_chunks = Layout::vertical([
            Constraint::Length(1), // name
            Constraint::Fill(1),   // description
        ])
        .split(chunks[0].inner(Margin::new(1, 0)));

        let current_reward = if self.owned_level == 0 {
            None
        } else {
            self.tiers.get(self.owned_level - 1).map(|t| t.reward)
        };

        let mut name_spans = vec![self.name.reset().bold()];
        if let Some(reward) = current_reward {
            name_spans.extend([
                " (currently ".dark_gray(),
                format_reward(reward).magenta(),
                ")".dark_gray(),
            ]);
        }

        Line::from(name_spans).render(name_chunks[0], buf);

        let description = Paragraph::new(self.description)
            .wrap(Wrap::default())
            .reset()
            .dim();
        description.render(name_chunks[1], buf);

        // cost info

        // vertical divider
        let block = Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().dark_gray());
        let inner = block.inner(chunks[1]);
        block.render(chunks[1], buf);

        let chunks = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);
        let middle = chunks[1];

        let next_tier = self.tiers.get(self.owned_level);
        let cost = next_tier.map(|t| t.cost).unwrap_or(0);
        let next_reward = next_tier
            .map(|t| t.reward)
            .unwrap_or_else(|| Reward::FlatPoints(0)); // fallback won't be used

        // text style: green/red for next based on affordability
        let affordable = self.state.party_points >= cost;
        let cost_style = if affordable { Color::Green } else { Color::Red };

        // if the bonus is not bought yet (current tier is zero),
        // show price to "buy"
        if self.owned_level == 0 {
            Line::from(vec![
                "Buy ".reset(),
                format_reward(next_reward).magenta(),
                " for ".reset().dim(),
                Span::from(format_cost(cost)).style(cost_style),
            ])
            .alignment(Alignment::Center)
            .render(middle, buf);
        }
        // if the bonus is already owned and is upgradeable,
        // show price to upgrade
        else if self.owned_level < self.tiers.len() {
            Line::from(vec![
                "Upgrade to ".reset(),
                format_reward(next_reward).magenta(),
                " for ".reset().dim(),
                Span::from(format_cost(cost)).style(cost_style),
            ])
            .alignment(Alignment::Center)
            .render(middle, buf);
        }
        // if the bonus is at max level, say so
        else {
            Line::from("Already at max level!")
                .cyan()
                .dim()
                .alignment(Alignment::Center)
                .render(middle, buf);
        }

        // // tiers: show current tier first, then next (gold), then rest
        // let tiers_area = chunks[2];
        // let max_visible = (tiers_area.width / TIER_WIDTH) as usize;

        // // start from current tier (last owned), or 0 if none owned
        // let start_idx = owned_level.saturating_sub(1);
        // let end_idx = (start_idx + max_visible).min(self.tiers.len());
        // let visible_count = end_idx - start_idx;

        // let tiers_constraints: Vec<_> = (0..visible_count)
        //     .map(|_| Constraint::Length(TIER_WIDTH))
        //     .collect();
        // let tiers_chunks = Layout::horizontal(tiers_constraints).split(tiers_area);

        // for (render_idx, idx) in (start_idx..end_idx).enumerate() {
        //     let tier = &self.tiers[idx];
        //     let tier_label = format_reward(tier.reward);
        //     let tier_cost = tier.cost;
        //     let is_current = idx == owned_level.saturating_sub(1) && owned_level > 0;
        //     let is_next = idx == owned_level && self.selected;
        //     let affordable = self.state.party_points >= tier_cost;

        //     // text style: white for current, green/red for next based on affordability
        //     let style = if is_current {
        //         Style::default().fg(Color::White).bold()
        //     } else if is_next {
        //         let color = if affordable { Color::Green } else { Color::Red };
        //         Style::default().fg(color).bold()
        //     } else {
        //         Style::default().fg(Color::DarkGray)
        //     };

        //     // border style: white for current, green/red for next, dark gray otherwise
        //     let border_style = if is_current {
        //         Style::default().fg(Color::White)
        //     } else if is_next {
        //         let color = if affordable { Color::Green } else { Color::Red };
        //         Style::default().fg(color)
        //     } else {
        //         Style::default().fg(Color::DarkGray)
        //     };

        //     let chunk = tiers_chunks[render_idx];

        //     let block = Block::default()
        //         .border_style(border_style)
        //         .borders(Borders::ALL);
        //     let inner = block.inner(chunk);
        //     block.render(chunk, buf);

        //     let inner_chunks =
        //         Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(inner);

        //     let label_text = Text::from(tier_label)
        //         .style(style)
        //         .alignment(Alignment::Center);
        //     label_text.render(inner_chunks[0], buf);

        //     let cost_display = if idx < owned_level {
        //         "✓".to_string()
        //     } else {
        //         format_cost(tier_cost)
        //     };
        //     let cost_text = Text::from(cost_display)
        //         .style(style)
        //         .alignment(Alignment::Center);
        //     cost_text.render(inner_chunks[1], buf);
        // }
    }
}

#[derive(Default)]
pub struct BonusesView {
    selection: usize,
    scroll_state: ScrollViewState,
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
        let content_width = content_area.width.saturating_sub(1); // leave room for scrollbar
        let content_height = ALL_TRACKS.len() as u16 * ITEM_HEIGHT;

        let mut scroll_view = ScrollView::new(Size::new(content_width, content_height))
            .horizontal_scrollbar_visibility(ScrollbarVisibility::Never);

        for (i, track) in ALL_TRACKS.iter().enumerate() {
            let owned_level = state.bonus_level(track.id()) as usize;
            let item = BonusItem::new(
                track.name(),
                track.description(),
                track.tiers(),
                state,
                self.selection == i,
                tick,
                owned_level,
            );
            let item_rect = Rect::new(0, i as u16 * ITEM_HEIGHT, content_width, ITEM_HEIGHT);
            scroll_view.render_widget(item, item_rect);
        }

        frame.render_stateful_widget(scroll_view, content_area, &mut self.scroll_state.clone());
    }

    fn handle(&mut self, action: Action, state: &mut State) -> ViewResult {
        match action {
            Action::Up => {
                let count = ALL_TRACKS.len();
                self.selection = (self.selection + count - 1) % count;
                self.update_scroll(20);
                ViewResult::Redraw
            }
            Action::Down => {
                self.selection = (self.selection + 1) % ALL_TRACKS.len();
                self.update_scroll(20);
                ViewResult::Redraw
            }
            Action::Select => {
                let track = ALL_TRACKS[self.selection];
                let current_level = state.bonus_level(track.id());
                let tiers = track.tiers();

                if current_level as usize >= tiers.len() {
                    return ViewResult::Message(
                        MessageType::Normal,
                        "Max level reached.".to_string(),
                    );
                }

                let next_tier = &tiers[current_level as usize];
                if state.party_points >= next_tier.cost {
                    state.party_points -= next_tier.cost;
                    state.set_bonus_level(track.id(), current_level + 1);
                    ViewResult::Message(MessageType::Success, format!("Upgraded {}!", track.name()))
                } else {
                    ViewResult::Message(MessageType::Error, "Not enough points.".to_string())
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

fn format_cost(cost: u64) -> String {
    if cost >= 1_000_000 {
        format!("{}M P", cost / 1_000_000)
    } else if cost >= 1_000 {
        format!("{}K P", cost / 1_000)
    } else {
        format!("{} P", cost)
    }
}

fn format_reward(reward: Reward) -> String {
    match reward {
        Reward::Multiplier(n) => format!("{}x", n),
        Reward::FlatPoints(n) => format!("+{}", n),
    }
}

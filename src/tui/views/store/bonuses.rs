use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::state::State;
use crate::tui::action::{Action, Route, StoreRoute};
use crate::tui::views::{MessageType, View, ViewResult};
use crate::tui::widgets::Card;

/// bonus track with tier-based upgrades
struct BonusTrack {
    name: &'static str,
    description: &'static str,
    tiers: &'static [(&'static str, u64)], // (label, cost) - cost 0 means owned
}

const BONUS_TRACKS: &[BonusTrack] = &[
    BonusTrack {
        name: "Commit Value",
        description: "How many party points you get per commit.",
        tiers: &[("1", 0), ("2", 0), ("3", 100), ("4", 1000), ("5", 10000)],
    },
    BonusTrack {
        name: "Weekend Warrior",
        description: "Earn more points for pushing code on Saturday or Sunday.",
        tiers: &[
            ("1x", 0),
            ("2x", 0),
            ("3x", 100),
            ("4x", 1000),
            ("5x", 10000),
        ],
    },
];

pub struct BonusesView {
    selection: usize,
    tier_selection: usize,
}

impl Default for BonusesView {
    fn default() -> Self {
        Self {
            selection: 0,
            tier_selection: 2, // default to first purchasable tier
        }
    }
}

impl View for BonusesView {
    fn render(&self, frame: &mut Frame, area: Rect, state: &State) {
        let mut constraints = vec![Constraint::Length(2)]; // sub-header
        for _ in BONUS_TRACKS {
            constraints.push(Constraint::Length(7));
        }
        constraints.push(Constraint::Min(0)); // spacer

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .margin(1)
            .split(area);

        // sub-header
        let header = Paragraph::new("Bonuses")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White));
        frame.render_widget(header, chunks[0]);

        // bonus tracks
        for (i, track) in BONUS_TRACKS.iter().enumerate() {
            let selected = self.selection == i;

            // determine owned level (mock: first 2 tiers are owned)
            let owned_level = if i == 0 {
                state.commit_value_level as usize
            } else {
                2 // mock for weekend warrior
            };

            // build tier display
            let tier_spans: Vec<Span> = track
                .tiers
                .iter()
                .enumerate()
                .flat_map(|(ti, &(label, _cost))| {
                    let is_owned = ti < owned_level;
                    let is_tier_selected = selected && ti == self.tier_selection;

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

                    vec![
                        Span::styled("[", border_style),
                        Span::styled(format!(" {} ", label), style),
                        Span::styled("]", border_style),
                        Span::raw("  "),
                    ]
                })
                .collect();

            // cost/status line
            let cost_spans: Vec<Span> = track
                .tiers
                .iter()
                .enumerate()
                .flat_map(|(ti, &(_, cost))| {
                    let is_owned = ti < owned_level;
                    let text = if is_owned {
                        "✓".to_string()
                    } else {
                        format_cost(cost)
                    };

                    let style = if is_owned {
                        Style::default().fg(Color::Green)
                    } else if state.party_points >= cost {
                        Style::default().fg(Color::White)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };

                    // pad to match tier width
                    let padded = format!("{:^5}", text);
                    vec![Span::styled(padded, style), Span::raw("  ")]
                })
                .collect();

            let card = Card::new()
                .title(track.name)
                .content(vec![
                    Line::from(track.description),
                    Line::from(""),
                    Line::from(tier_spans),
                    Line::from(cost_spans),
                ])
                .selected(selected);
            frame.render_widget(card, chunks[i + 1]);
        }
    }

    fn handle(&mut self, action: Action, _state: &mut State) -> ViewResult {
        match action {
            Action::Up => {
                let count = BONUS_TRACKS.len();
                self.selection = (self.selection + count - 1) % count;
                ViewResult::Redraw
            }
            Action::Down => {
                self.selection = (self.selection + 1) % BONUS_TRACKS.len();
                ViewResult::Redraw
            }
            Action::Left => {
                if self.tier_selection > 0 {
                    self.tier_selection -= 1;
                }
                ViewResult::Redraw
            }
            Action::Right => {
                let tier_count = BONUS_TRACKS[self.selection].tiers.len();
                if self.tier_selection < tier_count - 1 {
                    self.tier_selection += 1;
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
            ("Enter", "buy"),
            ("Esc", "back"),
            ("q", "quit"),
        ]
    }
}

fn format_cost(cost: u64) -> String {
    if cost >= 10000 {
        format!("{}K P", cost / 1000)
    } else if cost >= 1000 {
        format!("{}K P", cost / 1000)
    } else {
        format!("{} P", cost)
    }
}

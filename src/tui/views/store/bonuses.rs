use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

use crate::state::State;
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
}

impl<'a> BonusItem<'a> {
    fn new(track: BonusTrack, state: &'a State, selected: bool, tier_selection: usize) -> Self {
        Self {
            track,
            state,
            selected,
            tier_selection,
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
            Style::default().white()
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
        let title = Text::from(self.track.name).white().bold();
        title.render(chunks[0], buf);

        // description
        let description = Text::from(self.track.description).white();
        description.render(chunks[1], buf);

        // tiers
        let tiers_constraints = self
            .track
            .tiers
            .iter()
            .map(|_| Constraint::Length(10))
            .collect::<Vec<_>>();
        let tiers_chunks = Layout::horizontal(tiers_constraints).split(chunks[2]);

        // TODO: extract component
        for (idx, (tier_label, tier_cost)) in self.track.tiers.iter().enumerate() {
            let is_owned = idx < owned_level;
            let is_tier_selected = self.selected && idx == self.tier_selection;

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

            let chunk = tiers_chunks[idx];

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
            constraints.push(Constraint::Length(8));
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .horizontal_margin(1)
            .split(area);

        // sub-header
        let block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().dark_gray());
        let header = Paragraph::new("Bonuses")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White))
            .block(block);
        frame.render_widget(header, chunks[0]);

        // bonus tracks
        for (i, track) in BONUS_TRACKS.iter().enumerate() {
            let selected = self.selection == i;

            let item = BonusItem::new(track.clone(), state, selected, self.tier_selection);
            frame.render_widget(item, chunks[i + 1]);
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

use ratatui::prelude::*;

use crate::state::{PartyFeature, State, PARTY_FEATURES};
use crate::tui::views::MessageType;
use crate::tui::widgets::Card;

use super::{Action, Route, View, ViewResult};

pub struct PartyView {
    selection: usize,
}

impl Default for PartyView {
    fn default() -> Self {
        Self { selection: 0 }
    }
}

impl PartyView {
    fn item_count(&self) -> usize {
        // "Basic Party" (always enabled) + unlockable features
        1 + PARTY_FEATURES.len()
    }

    fn selected_feature(&self) -> Option<PartyFeature> {
        if self.selection == 0 {
            None // basic party, not toggleable
        } else {
            PARTY_FEATURES.get(self.selection - 1).copied()
        }
    }
}

impl View for PartyView {
    fn render(&self, frame: &mut Frame, area: Rect, state: &State) {
        let mut constraints: Vec<Constraint> = vec![Constraint::Length(5)]; // basic party
        for _ in PARTY_FEATURES {
            constraints.push(Constraint::Length(5));
        }
        constraints.push(Constraint::Min(0)); // spacer

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .margin(1)
            .split(area);

        // basic party (always enabled)
        let basic_card = Card::new()
            .title("Basic Party")
            .content(vec![
                Line::from("A simple summary of how many points you earned."),
                Line::from(""),
                Line::from(Span::styled("✓ Enabled", Style::default().fg(Color::Green))),
            ])
            .selected(self.selection == 0);
        frame.render_widget(basic_card, chunks[0]);

        // party features
        for (i, &feature) in PARTY_FEATURES.iter().enumerate() {
            let selected = self.selection == i + 1;
            let unlocked = state.is_unlocked(feature);
            let enabled = state.is_enabled(feature);

            let status_line = if !unlocked {
                Line::from(Span::styled(
                    "🔒 Locked",
                    Style::default().fg(Color::DarkGray),
                ))
            } else if enabled {
                Line::from(Span::styled("✓ Enabled", Style::default().fg(Color::Green)))
            } else {
                Line::from(Span::styled("✗ Disabled", Style::default().fg(Color::Red)))
            };

            let description = match feature {
                PartyFeature::Exclamations => "An excited message to hype up your party.",
                PartyFeature::Quotes => "An inspirational quote to motivate you.",
                PartyFeature::BigText => "Finish your party with a full screen word.",
            };

            let card = Card::new()
                .title(feature.name())
                .content(vec![Line::from(description), Line::from(""), status_line])
                .selected(selected);
            frame.render_widget(card, chunks[i + 1]);
        }
    }

    fn handle(&mut self, action: Action, state: &mut State) -> ViewResult {
        match action {
            Action::Up => {
                let count = self.item_count();
                self.selection = (self.selection + count - 1) % count;
                ViewResult::Redraw
            }
            Action::Down => {
                self.selection = (self.selection + 1) % self.item_count();
                ViewResult::Redraw
            }
            Action::Select => {
                if let Some(feature) = self.selected_feature() {
                    if state.is_unlocked(feature) {
                        state.toggle_feature(feature);
                        ViewResult::Redraw
                    } else {
                        ViewResult::Message(MessageType::Error, "Feature is locked".to_string())
                    }
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

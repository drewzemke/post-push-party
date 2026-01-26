use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::state::{feature_cost, PartyFeature, State, PARTY_FEATURES};
use crate::tui::action::{Action, Route, StoreRoute};
use crate::tui::views::{MessageType, View, ViewResult};
use crate::tui::widgets::Card;

pub struct UpgradesView {
    selection: usize,
}

impl Default for UpgradesView {
    fn default() -> Self {
        Self { selection: 0 }
    }
}

impl UpgradesView {
    fn selected_feature(&self) -> Option<PartyFeature> {
        PARTY_FEATURES.get(self.selection).copied()
    }
}

impl View for UpgradesView {
    fn render(&self, frame: &mut Frame, area: Rect, state: &State) {
        // initially split out a header that contains a dividing top line
        // and the title of the view
        let chunks = Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).split(area);

        let block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().dark_gray());
        let header = Paragraph::new("Upgrades")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White))
            .block(block);
        frame.render_widget(header, chunks[0]);

        let constraints = PARTY_FEATURES.iter().map(|_| Constraint::Length(5));

        let chunks = Layout::vertical(constraints)
            .horizontal_margin(1)
            .split(chunks[1]);

        // upgrade items
        for (i, &feature) in PARTY_FEATURES.iter().enumerate() {
            let selected = self.selection == i;
            let unlocked = state.is_unlocked(feature);
            let cost = feature_cost(feature);
            let affordable = state.party_points >= cost;

            let description = match feature {
                PartyFeature::Exclamations => "Adds an excited shout to your party.",
                PartyFeature::Quotes => "An inspirational quote after each push.",
                PartyFeature::BigText => "Finish your party with a full screen word. NICE!",
            };

            let price_style = if unlocked {
                Style::default().fg(Color::DarkGray)
            } else if affordable {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };

            let price_text = if unlocked {
                "✓ Owned".to_string()
            } else {
                format!("{} P", cost)
            };

            // title line with price right-aligned
            let title_width = feature.name().len();
            let price_width = price_text.len();
            let card_inner_width = (area.width as usize).saturating_sub(6); // margins + borders
            let spacing = card_inner_width.saturating_sub(title_width + price_width);

            let title_line = Line::from(vec![
                Span::styled(feature.name(), Style::default().fg(Color::White)),
                Span::raw(" ".repeat(spacing)),
                Span::styled(price_text, price_style),
            ]);

            let card = Card::new()
                .content(vec![title_line, Line::from(description)])
                .selected(selected);
            frame.render_widget(card, chunks[i]);
        }
    }

    fn handle(&mut self, action: Action, state: &mut State) -> ViewResult {
        match action {
            Action::Up => {
                let count = PARTY_FEATURES.len();
                self.selection = (self.selection + count - 1) % count;
                ViewResult::Redraw
            }
            Action::Down => {
                self.selection = (self.selection + 1) % PARTY_FEATURES.len();
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

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    pack::PackItem,
    party::{ALL_PARTIES, palette::ALL_PALETTES},
    state::State,
    tui::{
        action::{Action, Route},
        views::{View, ViewResult},
        widgets::palette_preview,
    },
};

enum PackItemState {
    Opened,
    Unopened,
}

const ITEM_HEIGHT: u16 = 5;
const ITEM_WIDTH: u16 = 10;
const ROW_SPACE_VERT: u16 = 1;
const ROW_SPACE_HORI: u16 = 6;

pub fn item_preview(item: &PackItem) -> Vec<Line<'static>> {
    match item {
        PackItem::PaletteUnlock { palette_id, .. } => {
            let palette = ALL_PALETTES
                .iter()
                .find(|p| p.id() == *palette_id)
                .expect("palette should exist");

            ["".into(), palette_preview(palette, false), "".into()].into()
        }
        PackItem::PointBundle { rarity, .. } => match rarity {
            crate::pack::Rarity::Common => ["".into(), "●●".yellow().into(), "".into()].into(),
            crate::pack::Rarity::Rare => [
                "".yellow().into(),
                "●●●●".yellow().into(),
                "".yellow().into(),
            ]
            .into(),
            crate::pack::Rarity::Epic => [
                "●●".yellow().into(),
                "●●●●".yellow().into(),
                "●●".yellow().into(),
            ]
            .into(),
            crate::pack::Rarity::Legendary => [
                "●●●●".yellow().into(),
                "●●●●●●".yellow().into(),
                "●●●●".yellow().into(),
            ]
            .into(),
        },
    }
}

#[derive(Default)]
pub struct PackRevealView {
    items: Vec<(PackItem, PackItemState)>,
    selected: Option<usize>,
}

impl PackRevealView {
    pub fn set_items(&mut self, items: Vec<PackItem>) {
        self.items = items
            .into_iter()
            .map(|i| (i, PackItemState::Unopened))
            .collect();
    }

    fn reset(&mut self) {
        self.items = Vec::new();
        self.selected = None;
    }
}

impl View for PackRevealView {
    fn render(&self, frame: &mut Frame, area: Rect, _state: &State, _tick: u32) {
        // strategy: show the pack items spread over two evenly-spaced rows
        // this will look nice for pack templates with 3, 5, 7, and maybe 9 items
        // this won't look as nice for templates with an even number or too many items
        let split_idx = self.items.len().div_ceil(2);
        let first_row_items = &self
            .items
            .iter()
            .enumerate()
            .take(split_idx)
            .collect::<Vec<_>>();
        let second_row_items = &self
            .items
            .iter()
            .enumerate()
            .skip(split_idx)
            .collect::<Vec<_>>();

        // IDEA: put a line in the middle that shows the name of the seleted pack (or something else if unopened)
        // inside each cell goes a three-symbol representation (yellow circles for points, palette preview, etc)
        // or just "???" when unopened
        let [_, first_row, _, middle, _, second_row, _] = area.layout(&Layout::vertical([
            Constraint::Fill(2),
            Constraint::Length(ITEM_HEIGHT),
            Constraint::Length(ROW_SPACE_VERT),
            Constraint::Length(1),
            Constraint::Length(ROW_SPACE_VERT),
            Constraint::Length(ITEM_HEIGHT),
            Constraint::Fill(2),
        ]));

        // middle text
        let selected_item = self.selected.and_then(|selected| {
            first_row_items
                .iter()
                .chain(second_row_items)
                .find(|(idx, _)| *idx == selected)
        });

        if let Some((_, (item, state))) = selected_item {
            let middle_text = match (item, state) {
                (
                    PackItem::PaletteUnlock {
                        party_id,
                        palette_id,
                        ..
                    },
                    PackItemState::Opened,
                ) => {
                    let party = ALL_PARTIES
                        .iter()
                        .find(|p| p.id() == *party_id)
                        .expect("party should exist");
                    let palette = ALL_PALETTES
                        .iter()
                        .find(|p| p.id() == *palette_id)
                        .expect("palette should exist");
                    Line::from(vec![
                        "You unlocked the ".dim(),
                        palette.name().bold(),
                        " palette for the ".dim(),
                        party.name().bold(),
                        "!".dim(),
                    ])
                }
                (PackItem::PointBundle { points, .. }, PackItemState::Opened) => Line::from(vec![
                    "You got ".into(),
                    points.yellow(),
                    " P".yellow(),
                    "!".into(),
                ]),
                (_, PackItemState::Unopened) => "Press enter to reveal!".dim().into(),
            };

            frame.render_widget(Text::from(middle_text.centered()), middle);
        } else {
            frame.render_widget(
                Text::from("(Use the arrow keys to navigate.)".dark_gray()).centered(),
                middle,
            );
        }

        // items
        for (row, items) in [(first_row, first_row_items), (second_row, second_row_items)] {
            // build layout
            let mut constraints = vec![Constraint::Fill(2)];
            for _ in 0..items.len().saturating_sub(1) {
                constraints.extend([
                    Constraint::Length(ITEM_WIDTH),
                    Constraint::Length(ROW_SPACE_HORI),
                ]);
            }
            constraints.extend([Constraint::Length(ITEM_WIDTH), Constraint::Fill(2)]);
            let layout = Layout::horizontal(constraints).split(row);

            for (rect, (idx, (item, state))) in layout.iter().skip(1).step_by(2).zip(items) {
                let selected = self.selected.is_some_and(|n| n == *idx);
                let opened = matches!(state, PackItemState::Opened);

                let block = if selected {
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().yellow())
                } else {
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().dark_gray())
                };

                let [_, middle, _] = rect.layout(&Layout::vertical([
                    Constraint::Fill(1),
                    Constraint::Length(3),
                    Constraint::Fill(1),
                ]));
                let text = if opened {
                    item_preview(item)
                } else {
                    ["".into(), "??".into(), "".into()].into()
                };

                let text = Paragraph::new(text).centered();

                frame.render_widget(block, *rect);
                frame.render_widget(text, middle);
            }
        }
    }

    fn handle(&mut self, action: Action, _state: &mut State) -> ViewResult {
        match action {
            Action::Up => {
                self.selected = if let Some(selected) = self.selected {
                    Some(selected.saturating_sub(1))
                } else {
                    Some(self.items.len() - 1)
                };
                ViewResult::Redraw
            }
            Action::Down => {
                self.selected = if let Some(selected) = self.selected {
                    Some((selected + 1).min(self.items.len() - 1))
                } else {
                    Some(0)
                };
                ViewResult::Redraw
            }
            Action::Right => ViewResult::None,
            Action::Left => ViewResult::None,
            Action::Select => {
                let selected_item = self.selected.and_then(|idx| self.items.get_mut(idx));
                if let Some((item, state)) = selected_item
                    && matches!(state, PackItemState::Unopened)
                {
                    *state = PackItemState::Opened;
                    if let PackItem::PointBundle { points, .. } = item {
                        ViewResult::RevealPoints(*points)
                    } else {
                        ViewResult::Redraw
                    }
                } else {
                    ViewResult::None
                }
            }
            Action::Back => {
                self.reset();
                ViewResult::Navigate(Route::Packs)
            }
            Action::Tab(i) => {
                self.reset();
                ViewResult::Navigate(match i {
                    0 => Route::Store(Default::default()),
                    1 => Route::Party,
                    2 => Route::Packs,
                    _ => Route::Games,
                })
            }
            Action::Quit => ViewResult::Exit,
            _ => ViewResult::None,
        }
    }

    fn key_hints(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("↑↓←→", "select"),
            ("enter", "reveal"),
            ("esc", "back"),
            ("q", "quit"),
        ]
    }
}

use ratatui::{prelude::*, widgets::List};

use crate::{
    pack::PackItem,
    state::State,
    tui::{
        action::{Action, Route},
        views::{View, ViewResult},
    },
};

enum PackItemState {
    Opened,
    Unopened,
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
        let items: Vec<_> = self
            .items
            .iter()
            .enumerate()
            .map(|(idx, (item, state))| {
                let selected = self.selected.is_some_and(|n| n == idx);
                let indicator = if selected { "> " } else { "  " };

                let pack_text = match state {
                    PackItemState::Opened => item.name(),
                    PackItemState::Unopened => "???".into(),
                };

                format!("{indicator}{pack_text}")
            })
            .collect();
        frame.render_widget(List::new(items), area);
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

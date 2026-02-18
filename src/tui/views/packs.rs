use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::state::State;

use super::{Action, Route, View, ViewResult};

#[derive(Default)]
pub struct PacksView;

impl View for PacksView {
    fn render(&self, frame: &mut Frame, area: Rect, _state: &State, _tick: u32) {
        let text = Paragraph::new("Packs coming soon...")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(text, area);
    }

    fn handle(&mut self, action: Action, _state: &mut State) -> ViewResult {
        match action {
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
        vec![("q", "quit")]
    }
}

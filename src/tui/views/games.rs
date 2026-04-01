use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::state::State;

use super::{Action, View, ViewResult};

#[derive(Default)]
pub struct GamesView;

impl View for GamesView {
    fn render(&self, frame: &mut Frame, area: Rect, _state: &State, _tick: u32) {
        let text = Paragraph::new("Games coming soon...")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(text, area);
    }

    fn handle(&mut self, _action: Action, _state: &mut State) -> ViewResult {
        ViewResult::None
    }

    fn key_hints(&self) -> Vec<(&'static str, &'static str)> {
        vec![("q", "quit")]
    }
}

pub mod games;
pub mod packs;
pub mod party;
pub mod store;

use ratatui::prelude::*;

use crate::state::State;

use super::action::{Action, Route};

/// result of handling an action in a view
pub enum ViewResult {
    /// nothing happened
    None,
    /// state changed, needs redraw
    Redraw,
    /// navigate to a different route
    Navigate(Route),
    /// show a transient message
    Message(String),
    /// exit the TUI
    Exit,
}

/// a renderable, interactive view
pub trait View {
    /// render the view's content (not header/footer, those are handled by App)
    fn render(&self, frame: &mut Frame, area: Rect, state: &State);

    /// handle an action, potentially mutating game state
    fn handle(&mut self, action: Action, state: &mut State) -> ViewResult;

    /// key hints for the footer, e.g. [("↑↓", "select"), ("Enter", "confirm")]
    fn key_hints(&self) -> Vec<(&'static str, &'static str)>;
}

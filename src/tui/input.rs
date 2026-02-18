use crossterm::event::{KeyCode, KeyEvent};

use super::action::Action;

/// maps key events to actions
pub fn map_key(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => Some(Action::Quit),
        KeyCode::Esc => Some(Action::Back),
        KeyCode::Enter | KeyCode::Char(' ') => Some(Action::Select),

        KeyCode::Up | KeyCode::Char('k') => Some(Action::Up),
        KeyCode::Down | KeyCode::Char('j') => Some(Action::Down),
        KeyCode::Left | KeyCode::Char('h') => Some(Action::Left),
        KeyCode::Right | KeyCode::Char('l') => Some(Action::Right),

        // FIXME: these should be mapped to separate actions,
        // don't use left/right for tabs
        KeyCode::Tab => Some(Action::Right),
        KeyCode::BackTab => Some(Action::Left),

        KeyCode::Char('p') => Some(Action::Palette),

        KeyCode::Char('1') => Some(Action::Tab(0)),
        KeyCode::Char('2') => Some(Action::Tab(1)),
        KeyCode::Char('3') => Some(Action::Tab(2)),
        KeyCode::Char('4') => Some(Action::Tab(3)),

        _ => None,
    }
}

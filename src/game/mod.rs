mod snake;

pub use snake::Snake;

use crate::tui::Terminal;

pub trait Game: Sync {
    // TODO: add state

    /// unique identifier for state storage
    fn id(&self) -> &'static str;

    /// display name for the UI
    fn name(&self) -> &'static str;

    /// description for the UI
    fn description(&self) -> &'static str;

    /// cost of a single game token
    fn cost(&self) -> u64;

    /// runs a game.
    ///
    /// suspends the normal party tui runs an entire separate tui for the game
    fn run(&self, terminal: &mut Terminal);
}

pub type GameRef = &'static dyn Game;

// static instances
pub static SNAKE: Snake = Snake;

// all parties in order
pub static ALL_GAMES: &[GameRef] = &[&SNAKE];

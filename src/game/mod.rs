use serde::{Serialize, de::DeserializeOwned};

use crate::tui::Terminal;

mod snake;
mod wallet;

pub use snake::Snake;
pub use wallet::GameWallet;

pub trait Game: Sync {
    type State: Serialize + DeserializeOwned + Default;

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
    fn run(
        &self,
        terminal: &mut Terminal,
        wallet: &GameWallet,
        state: &mut Self::State,
    ) -> anyhow::Result<()>;
}

/// an object-safe version of the `Game` trait, so we can put them all
/// into an array of dyn references
pub trait GameObject: Sync {
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
    /// suspends the normal party tui runs an entire separate tui for the game.
    /// handles tranforming the state blob to/from the game's `State` type before/after running
    fn run(
        &self,
        terminal: &mut Terminal,
        wallet: &GameWallet,
        state_json: &mut Option<String>,
    ) -> anyhow::Result<()>;
}

pub type GameRef = &'static dyn GameObject;

// blanket impl of `GameObject` for all `Game`s
impl<G: Game> GameObject for G {
    fn id(&self) -> &'static str {
        self.id()
    }

    fn name(&self) -> &'static str {
        self.name()
    }

    fn description(&self) -> &'static str {
        self.description()
    }

    fn cost(&self) -> u64 {
        self.cost()
    }

    fn run(
        &self,
        terminal: &mut Terminal,
        wallet: &GameWallet,
        state_json: &mut Option<String>,
    ) -> anyhow::Result<()> {
        // deserialize the state blob into the typed state object for this game
        let mut state: G::State = if let Some(state_json) = state_json {
            serde_json::from_str(state_json)?
        } else {
            G::State::default()
        };

        self.run(terminal, wallet, &mut state)?;

        // serialize and update
        *state_json = Some(serde_json::to_string(&state)?);

        Ok(())
    }
}

// static instances
pub static SNAKE: Snake = Snake;

// all parties in order
pub static ALL_GAMES: &[GameRef] = &[&SNAKE];

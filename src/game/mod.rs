use serde::{Serialize, de::DeserializeOwned};

use crate::tui::Terminal;

mod snake;
mod wallet;

pub use snake::Snake;
pub use wallet::{GameWallet, Wallet};

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

    /// the terminal will be transitioned towards this color before handing over
    /// render control to the game, and after the game finishes the terminal
    /// will transition from this color back to the Party TUI
    ///
    /// the game is responsible for gracefully transition to and from this
    /// color as it starts up and shuts down
    ///
    /// format is (r,g,b)
    fn clear_color(&self) -> (u8, u8, u8);

    /// runs a game.
    ///
    /// suspends the normal party tui runs an entire separate tui for the game
    fn run(
        &self,
        terminal: &mut Terminal,
        wallet: &'_ impl Wallet,
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

    /// format is (r,g,b)
    fn clear_color(&self) -> (u8, u8, u8);

    /// runs a game
    fn run(
        &self,
        terminal: &mut Terminal,
        wallet: &GameWallet,
        state_json: &mut Option<String>,
    ) -> anyhow::Result<()>;
}

impl std::fmt::Debug for dyn GameObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Game: {}", self.name())
    }
}

impl PartialEq for dyn GameObject {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
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

    fn clear_color(&self) -> (u8, u8, u8) {
        self.clear_color()
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

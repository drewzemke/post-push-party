use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "party", about = "earn party points by pushing code")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// install hook in current repo
    Init,
    /// remove hook from current repo
    Uninit,
    /// show current level and upgrade cost
    Status,
    /// called by git hook (not user-facing)
    #[command(hide = true)]
    Hook,
    /// snapshot current refs (called before push in jj)
    #[command(hide = true)]
    Snapshot,
    /// print state for debugging
    #[command(hide = true)]
    Dump,

    /// add or remove party points (dev only)
    #[cfg(feature = "dev")]
    #[command(allow_negative_numbers = true)]
    Cheat {
        /// amount to add (can be negative)
        amount: i64,
    },
    /// simulate pushing N commits (dev only)
    #[cfg(feature = "dev")]
    Push {
        /// number of commits to simulate
        commits: u64,
    },
    /// reset all state to defaults (dev only)
    #[cfg(feature = "dev")]
    Reset,
}

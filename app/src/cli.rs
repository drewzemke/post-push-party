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
    /// prints current party points
    Points,
    /// show push and commit stats
    Stats,
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
        /// lines changed per commit (comma-separated, cycles if fewer than commits)
        #[arg(long, value_delimiter = ',')]
        lines: Option<Vec<u64>>,
    },
    /// reset all state to defaults (dev only)
    #[cfg(feature = "dev")]
    Reset,
    /// set a bonus track to a specific level (dev only)
    #[cfg(feature = "dev")]
    Bonus {
        /// track id (e.g. "first_push")
        track: String,
        /// level to set (1 = first tier)
        level: u32,
    },
    /// unlock and enable a party (dev only)
    #[cfg(feature = "dev")]
    Party {
        /// party id (e.g. "big_text")
        id: String,
    },
}

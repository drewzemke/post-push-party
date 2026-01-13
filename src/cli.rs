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
    Init {
        /// branch to track (defaults to trunk)
        #[arg(long)]
        branch: Option<String>,
    },
    /// show current level and upgrade cost
    Status,
    /// buy the next commit value upgrade
    Upgrade,
    /// called by git hook (not user-facing)
    #[command(hide = true)]
    Hook,
    /// print state for debugging
    #[command(hide = true)]
    Dump,
}

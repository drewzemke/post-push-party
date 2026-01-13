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
    /// called by git hook (not user-facing)
    Hook,
    /// print state for debugging
    Dump,
}

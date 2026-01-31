mod bonus_tracks;
mod cli;
#[cfg(feature = "dev")]
mod dev;
mod git;
mod history;
mod hook;
mod init;
mod log;
mod party;
mod scoring;
mod state;
mod tui;

use clap::Parser;
use cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Init) => init::run(),
        Some(Command::Uninit) => init::run_uninit(),
        Some(Command::Status) => state::status(),
        Some(Command::Hook) => hook::post_push(),
        Some(Command::Dump) => state::dump(),
        Some(Command::Snapshot) => hook::pre_push(),

        None => tui::run().unwrap_or_else(|e| {
            eprintln!("error running TUI: {e}");
            std::process::exit(1);
        }),

        #[cfg(feature = "dev")]
        Some(Command::Cheat { amount }) => dev::cheat(amount),
        #[cfg(feature = "dev")]
        Some(Command::Push { commits, lines }) => dev::push(commits, lines),
        #[cfg(feature = "dev")]
        Some(Command::Reset) => dev::reset(),
        #[cfg(feature = "dev")]
        Some(Command::Unlock { track, level }) => dev::unlock(&track, level),
    }
}

mod cli;
#[cfg(feature = "dev")]
mod dev;
mod hook;
mod init;
mod log;
mod party;
mod state;
mod tui;

use clap::Parser;
use cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Init { branch }) => init::run(branch),
        Some(Command::Uninit) => init::run_uninit(),
        Some(Command::Status) => state::status(),
        Some(Command::Hook) => hook::run(),
        Some(Command::Dump) => state::dump(),
        None => tui::run().unwrap_or_else(|e| {
            eprintln!("error running TUI: {e}");
            std::process::exit(1);
        }),

        #[cfg(feature = "dev")]
        Some(Command::Cheat { amount }) => dev::cheat(amount),
        #[cfg(feature = "dev")]
        Some(Command::Push { commits }) => dev::push(commits),
        #[cfg(feature = "dev")]
        Some(Command::Reset) => dev::reset(),
    }
}

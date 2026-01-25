mod cli;
#[cfg(feature = "dev")]
mod dev;
mod git;
mod history;
mod hook;
mod init;
mod log;
mod party;
mod patch_ids;
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
        Some(Command::Hook) => {
            if let Some(push) = hook::run() {
                let mut s = state::load();

                let points_earned = push.commits * s.points_per_commit();
                s.party_points += points_earned;

                if let Err(e) = state::save(&s) {
                    eprintln!("warning: could not save state: {e}");
                }

                party::display(&s, push.commits, points_earned);
            }
        }
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

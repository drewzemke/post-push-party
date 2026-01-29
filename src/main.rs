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
mod patch_ids;
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
        Some(Command::Hook) => {
            if let Some(push) = hook::run() {
                let mut s = state::load();
                let history = history::load();
                let clock = scoring::now();

                let breakdown = scoring::calculate_points(
                    push.commits_counted,
                    &s,
                    &history,
                    &clock,
                );
                s.party_points += breakdown.total;

                if let Err(e) = state::save(&s) {
                    eprintln!("warning: could not save state: {e}");
                }

                party::display(&breakdown);
            }
        }
        Some(Command::Dump) => state::dump(),
        Some(Command::Snapshot) => {
            let cwd = std::env::current_dir().expect("could not get current directory");
            hook::snapshot_refs(&cwd);
        }
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
        #[cfg(feature = "dev")]
        Some(Command::Unlock { track, level }) => dev::unlock(&track, level),
    }
}

mod cli;
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
        Some(Command::Init { branch }) => cmd_init(branch),
        Some(Command::Uninit) => cmd_uninit(),
        Some(Command::Status) => cmd_status(),
        Some(Command::Hook) => cmd_hook(),
        Some(Command::Dump) => cmd_dump(),
        #[cfg(feature = "dev")]
        Some(Command::Cheat { amount }) => cmd_cheat(amount),
        None => cmd_points(),
    }
}

#[cfg(feature = "dev")]
fn cmd_cheat(amount: i64) {
    let mut state = state::load();
    let old = state.party_points;
    if amount < 0 {
        state.party_points = state.party_points.saturating_sub(amount.unsigned_abs());
    } else {
        state.party_points = state.party_points.saturating_add(amount as u64);
    }
    if let Err(e) = state::save(&state) {
        eprintln!("error saving state: {e}");
        std::process::exit(1);
    }
    println!("{} → {} party points", old, state.party_points);
}

fn cmd_points() {
    if let Err(e) = tui::run() {
        eprintln!("error running TUI: {e}");
        std::process::exit(1);
    }
}

fn cmd_status() {
    let state = state::load();
    println!("You have {} party points.", state.party_points);
}

const STARTER_POINTS: u64 = 10;

fn cmd_init(_branch: Option<String>) {
    let cwd = std::env::current_dir().expect("could not get current directory");

    match init::detect_repo_type(&cwd) {
        Some(init::RepoType::Jj) => {
            if let Err(e) = init::install_jj_alias(&cwd) {
                eprintln!("error installing jj alias: {e}");
                std::process::exit(1);
            }
            println!("installed jj push alias");
            println!("use `jj push` instead of `jj git push` to earn party points!");
        }
        Some(init::RepoType::Git) => {
            if let Err(e) = init::install_git_hook(&cwd) {
                eprintln!("error installing git hook: {e}");
                std::process::exit(1);
            }
            println!("installed git reference-transaction hook");
            println!("push code to earn party points!");
        }
        None => {
            eprintln!("not a git or jj repository");
            std::process::exit(1);
        }
    }

    // give starter points on first init
    let mut s = state::load();
    if s == state::State::default() {
        s.party_points = STARTER_POINTS;
        let _ = state::save(&s);
        println!();
        println!("🎁 You got {} starter party points!", STARTER_POINTS);
        println!("Run `party` to spend them!");
    }
}

fn cmd_uninit() {
    let cwd = std::env::current_dir().expect("could not get current directory");

    let result = match init::detect_repo_type(&cwd) {
        Some(init::RepoType::Jj) => init::uninstall_jj_alias(&cwd),
        Some(init::RepoType::Git) => init::uninstall_git_hook(&cwd),
        None => {
            eprintln!("not a git or jj repository");
            std::process::exit(1);
        }
    };

    match result {
        Ok(init::UninstallResult::Removed) => {
            println!("removed party hook");
        }
        Ok(init::UninstallResult::NotInstalled) => {
            println!("party hook not installed in this repo");
        }
        Ok(init::UninstallResult::ManualRemovalRequired) => {
            eprintln!("hook has been modified, please remove manually");
            match init::detect_repo_type(&cwd) {
                Some(init::RepoType::Jj) => {
                    eprintln!("  edit: {}", init::jj_config_path(&cwd).display());
                }
                Some(init::RepoType::Git) => {
                    eprintln!("  edit: {}", init::git_hook_path(&cwd).display());
                }
                _ => {}
            }
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("error removing hook: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_hook() {
    let cwd = std::env::current_dir().expect("could not get current directory");

    if let Some(result) = hook::detect_push(&cwd) {
        // award points
        let mut state = state::load();
        state.party_points += result.points_earned;
        if let Err(e) = state::save(&state) {
            eprintln!("warning: could not save state: {e}");
        }

        // show party based on unlocked level
        party::display(&state, result.commits, result.points_earned);
    }
}

fn cmd_dump() {
    let state = state::load();
    println!("party_points: {}", state.party_points);
    println!("commit_value_level: {}", state.commit_value_level);
    println!("points_per_commit: {}", state.points_per_commit());
    println!("upgrade_cost: {}", state.upgrade_cost());
}

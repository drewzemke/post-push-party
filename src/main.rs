mod cli;
mod hook;
mod init;
mod state;

use clap::Parser;
use cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Init { branch }) => cmd_init(branch),
        Some(Command::Status) => cmd_status(),
        Some(Command::Upgrade) => cmd_upgrade(),
        Some(Command::Hook) => cmd_hook(),
        Some(Command::Dump) => cmd_dump(),
        None => cmd_points(),
    }
}

fn cmd_points() {
    let state = state::load();
    println!("🎉 You have {} party points!", state.party_points);
}

fn cmd_status() {
    let state = state::load();
    println!("party points: {}", state.party_points);
    println!();
    println!("commit value: {} (level {})", state.points_per_commit(), state.commit_value_level);
    println!("next upgrade: {} points", state.upgrade_cost());
}

fn cmd_upgrade() {
    let mut state = state::load();
    let cost = state.upgrade_cost();

    if state.party_points < cost {
        println!("not enough points!");
        println!("you have {} points, but the upgrade costs {}", state.party_points, cost);
        std::process::exit(1);
    }

    state.party_points -= cost;
    state.commit_value_level += 1;

    if let Err(e) = state::save(&state) {
        eprintln!("error saving state: {e}");
        std::process::exit(1);
    }

    println!("🎉 Upgraded commit value to {}!", state.points_per_commit());
    println!("each commit now earns {} points", state.points_per_commit());
    println!();
    println!("remaining points: {}", state.party_points);
}

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

        // base party
        println!();
        println!("🎉 You earned {} party points!", result.points_earned);
        if result.commits > 1 {
            println!("   ({} commits × {} points each)", result.commits, state.points_per_commit());
        }
        println!();
        println!("Run `party` to see your total!");
        println!();
    }
}

fn cmd_dump() {
    let state = state::load();
    println!("party_points: {}", state.party_points);
    println!("commit_value_level: {}", state.commit_value_level);
    println!("points_per_commit: {}", state.points_per_commit());
    println!("upgrade_cost: {}", state.upgrade_cost());
}

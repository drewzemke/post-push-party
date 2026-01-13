mod cli;
mod state;

use clap::Parser;
use cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Init { branch }) => cmd_init(branch),
        Some(Command::Hook) => cmd_hook(),
        Some(Command::Dump) => cmd_dump(),
        None => cmd_status(),
    }
}

fn cmd_status() {
    let state = state::load();
    println!("🎉 You have {} party points!", state.party_points);
}

fn cmd_init(_branch: Option<String>) {
    todo!("init")
}

fn cmd_hook() {
    todo!("hook")
}

fn cmd_dump() {
    let state = state::load();
    println!("party_points: {}", state.party_points);
    println!("commit_value_level: {}", state.commit_value_level);
    println!("points_per_commit: {}", state.points_per_commit());
    println!("upgrade_cost: {}", state.upgrade_cost());
}

mod cli;
mod init;
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
    todo!("hook")
}

fn cmd_dump() {
    let state = state::load();
    println!("party_points: {}", state.party_points);
    println!("commit_value_level: {}", state.commit_value_level);
    println!("points_per_commit: {}", state.points_per_commit());
    println!("upgrade_cost: {}", state.upgrade_cost());
}

mod bonus_track;
mod cli;
mod clock;
#[cfg(feature = "dev")]
mod dev;
mod game;
mod git;
mod hook;
mod init;
mod pack;
mod party;
mod scoring;
mod state;
mod storage;
mod tui;

use clap::Parser;
use cli::{Cli, Command};

use crate::{
    state::State,
    storage::{BranchRefsStore, DbConnection, PatchIdStore, PushHistory},
};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // load state from sqlite db
    let conn = DbConnection::create()?;
    let mut state = State::load(&conn)?;
    let branch_refs = BranchRefsStore::new(&conn);
    let history = PushHistory::new(&conn);
    let patch_ids = PatchIdStore::new(&conn);

    match cli.command {
        Some(Command::Init) => init::run(&mut state, &branch_refs)?,
        Some(Command::Uninit) => init::run_uninit()?,
        Some(Command::Points) => state::points(&state),
        Some(Command::Stats) => state::stats(&state, &history),
        Some(Command::Hook) => hook::post_push(&mut state, &branch_refs, &history, &patch_ids)?,
        Some(Command::Dump) => state::dump(&state),
        Some(Command::Snapshot) => hook::pre_push(&branch_refs)?,

        None => tui::run(&mut state, &conn)?,

        #[cfg(feature = "dev")]
        Some(Command::Cheat { amount }) => dev::cheat(amount, &mut state),
        #[cfg(feature = "dev")]
        Some(Command::Push { commits, lines }) => dev::push(commits, lines, &mut state, &history)?,
        #[cfg(feature = "dev")]
        Some(Command::Reset) => dev::reset(&mut state, &history)?,
        #[cfg(feature = "dev")]
        Some(Command::Bonus { track, level }) => dev::bonus(&track, level, &mut state),
        #[cfg(feature = "dev")]
        Some(Command::Party { id }) => dev::party(&id, &mut state),
        #[cfg(feature = "dev")]
        Some(Command::Palette { id }) => dev::palette(&id, &mut state),
    }

    state.save(&conn)?;

    Ok(())
}

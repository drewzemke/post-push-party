use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};

use crate::{
    git,
    state::{self, State},
    storage::BranchRefsStore,
};

const STARTER_POINTS: u64 = 10;

#[derive(Debug, Clone, PartialEq)]
pub enum RepoType {
    Git,
    Jj,
}

pub const GIT_HOOK_SCRIPT: &str = r#"#!/bin/sh
if [ "$1" = "committed" ]; then
    party hook
fi
"#;

pub fn run(state: &mut State, branch_refs: &BranchRefsStore<'_>) -> Result<()> {
    let cwd = std::env::current_dir().context("could not get current directory")?;

    install_party_hook(&cwd).context("could not install party hook")?;
    println!("installed party hook");
    println!("push code to earn party points!");

    // snapshot current refs so we don't credit pre-existing commits
    git::snapshot_refs(&cwd, branch_refs)?;

    // give starter points on first init
    if *state == state::State::default() {
        state.party_points = STARTER_POINTS;
        println!();
        println!("🎁 You got {} starter party points!", STARTER_POINTS);
        println!("Run `party` to spend them!");
    }

    Ok(())
}

pub fn run_uninit() -> Result<()> {
    let cwd = std::env::current_dir().context("could not get current directory")?;

    uninstall_party_hook(&cwd).context("could not uninstall party hook")?;

    println!("removed party hook");

    Ok(())
}

pub fn detect_repo_type(cwd: &Path) -> Option<RepoType> {
    let jj_dir = cwd.join(".jj");
    let git_dir = cwd.join(".git");

    if jj_dir.is_dir() {
        Some(RepoType::Jj)
    } else if git_dir.is_dir() {
        Some(RepoType::Git)
    } else {
        None
    }
}

fn git_store_path(cwd: &Path) -> Result<PathBuf> {
    match detect_repo_type(cwd) {
        Some(RepoType::Git) => Ok(cwd.join(".git")),
        Some(RepoType::Jj) => {
            let path_to_git_target = jj_git_target_path(cwd);
            let path_to_store_from_target_dir = std::fs::read_to_string(path_to_git_target)?;
            let path_to_git_store =
                jj_repo_store_path(cwd).join(path_to_store_from_target_dir.trim());
            Ok(path_to_git_store.canonicalize()?)
        }
        None => Err(anyhow!("not a git or jj repository")),
    }
}

fn git_hook_path(cwd: &Path) -> Result<PathBuf> {
    let git_store_path = git_store_path(cwd)?;
    Ok(git_store_path.join("hooks/reference-transaction"))
}

fn jj_git_target_path(repo_path: &Path) -> PathBuf {
    jj_repo_store_path(repo_path).join("git_target")
}

fn jj_repo_store_path(repo_path: &Path) -> PathBuf {
    repo_path.join(".jj/repo/store")
}

pub fn install_party_hook(cwd: &Path) -> Result<()> {
    let path = git_hook_path(cwd)?;

    if path.exists() {
        return Err(anyhow!(
            "git hook already exists at {}\nto install party in this repo, please add the following to your existing hook:\n\n{GIT_HOOK_SCRIPT}",
            path.display()
        ));
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, GIT_HOOK_SCRIPT)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))?;
    }

    Ok(())
}

pub fn uninstall_party_hook(cwd: &Path) -> Result<()> {
    let path = git_hook_path(cwd)?;

    if !path.exists() {
        return Err(anyhow!("party hook not installed in this repo"));
    }

    let content = std::fs::read_to_string(&path)?;
    if content == GIT_HOOK_SCRIPT {
        std::fs::remove_file(path)?;
        Ok(())
    } else {
        Err(anyhow!(
            "hook has been modified, please remove manually\nedit: {}",
            path.display()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, process::Command};
    use tempfile::tempdir;

    fn git_init(cwd: &Path) -> Result<()> {
        let _ = Command::new("git").arg("init").current_dir(cwd).output()?;
        Ok(())
    }

    fn jj_init(cwd: &Path) -> Result<()> {
        let _ = Command::new("jj")
            .arg("git")
            .arg("init")
            .arg("--colocate")
            .current_dir(cwd)
            .output()?;
        Ok(())
    }

    fn jj_init_no_colocate(cwd: &Path) -> Result<()> {
        let _ = Command::new("jj")
            .arg("git")
            .arg("init")
            .arg("--no-colocate")
            .current_dir(cwd)
            .output()?;
        Ok(())
    }

    #[test]
    fn detect_git_repo() {
        let dir = tempdir().unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();

        assert_eq!(detect_repo_type(dir.path()), Some(RepoType::Git));
    }

    #[test]
    fn detect_jj_repo() {
        let dir = tempdir().unwrap();
        fs::create_dir(dir.path().join(".jj")).unwrap();

        assert_eq!(detect_repo_type(dir.path()), Some(RepoType::Jj));
    }

    #[test]
    fn jj_takes_precedence_over_git() {
        let dir = tempdir().unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();
        fs::create_dir(dir.path().join(".jj")).unwrap();

        assert_eq!(detect_repo_type(dir.path()), Some(RepoType::Jj));
    }

    #[test]
    fn detect_no_repo() {
        let dir = tempdir().unwrap();
        assert_eq!(detect_repo_type(dir.path()), None);
    }

    #[test]
    fn init_creates_hook_git() {
        let dir = tempdir().unwrap();
        git_init(dir.path()).unwrap();

        install_party_hook(dir.path()).unwrap();

        let content = fs::read_to_string(git_hook_path(dir.path()).unwrap()).unwrap();
        assert!(content.contains("party hook"));
    }

    #[test]
    fn init_creates_hook_jj() {
        let dir = tempdir().unwrap();
        jj_init(dir.path()).unwrap();

        install_party_hook(dir.path()).unwrap();

        let content = fs::read_to_string(git_hook_path(dir.path()).unwrap()).unwrap();
        assert!(content.contains("party hook"));
    }

    #[test]
    fn init_creates_hook_jj_no_colocate() {
        let dir = tempdir().unwrap();
        jj_init_no_colocate(dir.path()).unwrap();

        install_party_hook(dir.path()).unwrap();

        let content = fs::read_to_string(git_hook_path(dir.path()).unwrap()).unwrap();
        assert!(content.contains("party hook"));
    }

    #[test]
    fn init_when_hook_exists_git() {
        let dir = tempdir().unwrap();
        git_init(dir.path()).unwrap();

        // write a modified hook
        let content = "#!/bin/sh\nsome_command\nsome_other_command";
        fs::write(git_hook_path(dir.path()).unwrap(), content).unwrap();

        let result = install_party_hook(dir.path());
        assert!(result.is_err());

        let new_content = fs::read_to_string(git_hook_path(dir.path()).unwrap()).unwrap();
        assert_eq!(new_content, content);
    }

    #[test]
    fn init_when_hook_exists_jj() {
        let dir = tempdir().unwrap();
        jj_init(dir.path()).unwrap();

        // write a modified hook
        let content = "#!/bin/sh\nsome_command\nsome_other_command";
        fs::write(git_hook_path(dir.path()).unwrap(), content).unwrap();

        let result = install_party_hook(dir.path());
        assert!(result.is_err());

        let new_content = fs::read_to_string(git_hook_path(dir.path()).unwrap()).unwrap();
        assert_eq!(new_content, content);
    }

    #[test]
    fn init_when_hook_exists_jj_no_colocate() {
        let dir = tempdir().unwrap();
        jj_init_no_colocate(dir.path()).unwrap();

        // write a modified hook
        let content = "#!/bin/sh\nsome_command\nsome_other_command";
        fs::write(git_hook_path(dir.path()).unwrap(), content).unwrap();

        let result = install_party_hook(dir.path());
        assert!(result.is_err());

        let new_content = fs::read_to_string(git_hook_path(dir.path()).unwrap()).unwrap();
        assert_eq!(new_content, content);
    }

    #[test]
    fn uninit_removes_hook_git() {
        let dir = tempdir().unwrap();
        git_init(dir.path()).unwrap();

        install_party_hook(dir.path()).unwrap();
        assert!(git_hook_path(dir.path()).unwrap().exists());

        uninstall_party_hook(dir.path()).unwrap();
        assert!(!git_hook_path(dir.path()).unwrap().exists());
    }

    #[test]
    fn uninit_removes_hook_jj() {
        let dir = tempdir().unwrap();
        jj_init(dir.path()).unwrap();

        install_party_hook(dir.path()).unwrap();
        assert!(git_hook_path(dir.path()).unwrap().exists());

        uninstall_party_hook(dir.path()).unwrap();
        assert!(!git_hook_path(dir.path()).unwrap().exists());
    }

    #[test]
    fn uninit_removes_hook_jj_no_colocate() {
        let dir = tempdir().unwrap();
        jj_init_no_colocate(dir.path()).unwrap();

        install_party_hook(dir.path()).unwrap();
        assert!(git_hook_path(dir.path()).unwrap().exists());

        uninstall_party_hook(dir.path()).unwrap();
        assert!(!git_hook_path(dir.path()).unwrap().exists());
    }

    #[test]
    fn uninit_when_modified_git() {
        let dir = tempdir().unwrap();
        git_init(dir.path()).unwrap();

        install_party_hook(dir.path()).unwrap();
        assert!(git_hook_path(dir.path()).unwrap().exists());

        // write a modified hook
        fs::write(
            git_hook_path(dir.path()).unwrap(),
            "#!/bin/sh\nparty hook\nsome_other_command\n",
        )
        .unwrap();

        let result = uninstall_party_hook(dir.path());
        assert!(result.is_err());

        assert!(git_hook_path(dir.path()).unwrap().exists());
    }

    #[test]
    fn uninit_when_modified_jj() {
        let dir = tempdir().unwrap();
        jj_init(dir.path()).unwrap();

        install_party_hook(dir.path()).unwrap();
        assert!(git_hook_path(dir.path()).unwrap().exists());

        // write a modified hook
        fs::write(
            git_hook_path(dir.path()).unwrap(),
            "#!/bin/sh\nparty hook\nsome_other_command\n",
        )
        .unwrap();

        let result = uninstall_party_hook(dir.path());
        assert!(result.is_err());

        assert!(git_hook_path(dir.path()).unwrap().exists());
    }

    #[test]
    fn uninit_when_modified_jj_no_colocate() {
        let dir = tempdir().unwrap();
        jj_init_no_colocate(dir.path()).unwrap();

        install_party_hook(dir.path()).unwrap();
        assert!(git_hook_path(dir.path()).unwrap().exists());

        // write a modified hook
        fs::write(
            git_hook_path(dir.path()).unwrap(),
            "#!/bin/sh\nparty hook\nsome_other_command\n",
        )
        .unwrap();

        let result = uninstall_party_hook(dir.path());
        assert!(result.is_err());

        assert!(git_hook_path(dir.path()).unwrap().exists());
    }
}

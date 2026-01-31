use std::path::Path;

use crate::{git, state};

const STARTER_POINTS: u64 = 10;

#[derive(Debug, Clone, PartialEq)]
pub enum RepoType {
    Git,
    Jj,
}

pub fn run() {
    let cwd = std::env::current_dir().expect("could not get current directory");

    match detect_repo_type(&cwd) {
        Some(RepoType::Jj) => {
            if let Err(e) = install_jj_alias(&cwd) {
                eprintln!("error installing jj alias: {e}");
                std::process::exit(1);
            }
            println!("installed jj push alias");
            println!("use `jj push` instead of `jj git push` to earn party points!");
        }
        Some(RepoType::Git) => {
            if let Err(e) = install_git_hook(&cwd) {
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

    // snapshot current refs so we don't credit pre-existing commits
    git::snapshot_refs(&cwd);

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

pub fn run_uninit() {
    let cwd = std::env::current_dir().expect("could not get current directory");

    let result = match detect_repo_type(&cwd) {
        Some(RepoType::Jj) => uninstall_jj_alias(&cwd),
        Some(RepoType::Git) => uninstall_git_hook(&cwd),
        None => {
            eprintln!("not a git or jj repository");
            std::process::exit(1);
        }
    };

    match result {
        Ok(UninstallResult::Removed) => {
            println!("removed party hook");
        }
        Ok(UninstallResult::NotInstalled) => {
            println!("party hook not installed in this repo");
        }
        Ok(UninstallResult::ManualRemovalRequired) => {
            eprintln!("hook has been modified, please remove manually");
            match detect_repo_type(&cwd) {
                Some(RepoType::Jj) => {
                    eprintln!("  edit: {}", jj_config_path(&cwd).display());
                }
                Some(RepoType::Git) => {
                    eprintln!("  edit: {}", git_hook_path(&cwd).display());
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

pub fn detect_repo_type(path: &Path) -> Option<RepoType> {
    let jj_dir = path.join(".jj");
    let git_dir = path.join(".git");

    if jj_dir.is_dir() {
        Some(RepoType::Jj)
    } else if git_dir.is_dir() {
        Some(RepoType::Git)
    } else {
        None
    }
}

pub const GIT_HOOK_SCRIPT: &str = r#"#!/bin/sh
# only run after refs are committed, not on prepare or abort
if [ "$1" = "committed" ]; then
    party hook
fi
"#;

pub fn git_hook_path(repo_path: &Path) -> std::path::PathBuf {
    repo_path.join(".git/hooks/reference-transaction")
}

pub fn jj_config_path(repo_path: &Path) -> std::path::PathBuf {
    repo_path.join(".jj/repo/config.toml")
}

pub const JJ_ALIAS: &str = r#"[aliases]
push = ["util", "exec", "--", "bash", "-c", "party snapshot && jj git push \"$@\" && party hook", "--"]
"#;

pub fn install_git_hook(repo_path: &Path) -> std::io::Result<()> {
    let path = git_hook_path(repo_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, GIT_HOOK_SCRIPT)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
    }
    Ok(())
}

pub fn install_jj_alias(repo_path: &Path) -> std::io::Result<()> {
    let path = jj_config_path(repo_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, JJ_ALIAS)
}

#[derive(Debug)]
pub enum UninstallResult {
    Removed,
    NotInstalled,
    ManualRemovalRequired,
}

pub fn uninstall_git_hook(repo_path: &Path) -> std::io::Result<UninstallResult> {
    let path = git_hook_path(repo_path);

    if !path.exists() {
        return Ok(UninstallResult::NotInstalled);
    }

    let content = std::fs::read_to_string(&path)?;
    if content == GIT_HOOK_SCRIPT {
        std::fs::remove_file(&path)?;
        Ok(UninstallResult::Removed)
    } else {
        Ok(UninstallResult::ManualRemovalRequired)
    }
}

pub fn uninstall_jj_alias(repo_path: &Path) -> std::io::Result<UninstallResult> {
    let path = jj_config_path(repo_path);

    if !path.exists() {
        return Ok(UninstallResult::NotInstalled);
    }

    let content = std::fs::read_to_string(&path)?;
    if content == JJ_ALIAS {
        std::fs::remove_file(&path)?;
        Ok(UninstallResult::Removed)
    } else {
        Ok(UninstallResult::ManualRemovalRequired)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

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
    fn git_hook_creates_new_file() {
        let dir = tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        let hooks_dir = git_dir.join("hooks");
        fs::create_dir_all(&hooks_dir).unwrap();

        install_git_hook(dir.path()).unwrap();

        let content = fs::read_to_string(git_hook_path(dir.path())).unwrap();
        assert!(content.contains("party hook"));
    }

    #[test]
    fn jj_alias_creates_config() {
        let dir = tempdir().unwrap();
        let jj_repo_dir = dir.path().join(".jj/repo");
        fs::create_dir_all(&jj_repo_dir).unwrap();

        install_jj_alias(dir.path()).unwrap();

        let content = fs::read_to_string(jj_config_path(dir.path())).unwrap();
        assert!(content.contains("[aliases]"));
        assert!(content.contains("party hook"));
    }

    #[test]
    fn uninstall_git_hook_removes_party_hook() {
        let dir = tempdir().unwrap();
        let hooks_dir = dir.path().join(".git/hooks");
        fs::create_dir_all(&hooks_dir).unwrap();

        install_git_hook(dir.path()).unwrap();
        assert!(git_hook_path(dir.path()).exists());

        let result = uninstall_git_hook(dir.path()).unwrap();
        assert!(matches!(result, UninstallResult::Removed));
        assert!(!git_hook_path(dir.path()).exists());
    }

    #[test]
    fn uninstall_git_hook_not_installed() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git/hooks")).unwrap();

        let result = uninstall_git_hook(dir.path()).unwrap();
        assert!(matches!(result, UninstallResult::NotInstalled));
    }

    #[test]
    fn uninstall_git_hook_manual_removal() {
        let dir = tempdir().unwrap();
        let hooks_dir = dir.path().join(".git/hooks");
        fs::create_dir_all(&hooks_dir).unwrap();

        // write a modified hook
        fs::write(
            git_hook_path(dir.path()),
            "#!/bin/sh\nparty hook\nsome_other_command\n",
        )
        .unwrap();

        let result = uninstall_git_hook(dir.path()).unwrap();
        assert!(matches!(result, UninstallResult::ManualRemovalRequired));
        assert!(git_hook_path(dir.path()).exists());
    }

    #[test]
    fn uninstall_jj_alias_removes_config() {
        let dir = tempdir().unwrap();
        let jj_repo_dir = dir.path().join(".jj/repo");
        fs::create_dir_all(&jj_repo_dir).unwrap();

        install_jj_alias(dir.path()).unwrap();
        assert!(jj_config_path(dir.path()).exists());

        let result = uninstall_jj_alias(dir.path()).unwrap();
        assert!(matches!(result, UninstallResult::Removed));
        assert!(!jj_config_path(dir.path()).exists());
    }

    #[test]
    fn uninstall_jj_alias_not_installed() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".jj/repo")).unwrap();

        let result = uninstall_jj_alias(dir.path()).unwrap();
        assert!(matches!(result, UninstallResult::NotInstalled));
    }
}

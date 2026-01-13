use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum RepoType {
    Git,
    Jj,
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
party hook "$@"
"#;

pub fn git_hook_path(repo_path: &Path) -> std::path::PathBuf {
    repo_path.join(".git/hooks/reference-transaction")
}

pub fn jj_config_path(repo_path: &Path) -> std::path::PathBuf {
    repo_path.join(".jj/repo/config.toml")
}

pub const JJ_ALIAS: &str = r#"[aliases]
push = ["util", "exec", "--", "bash", "-c", "jj git push \"$@\" && party hook", "--"]
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
        fs::write(git_hook_path(dir.path()), "#!/bin/sh\nparty hook\nsome_other_command\n").unwrap();

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

use std::path::Path;
use std::process::Command;

pub fn get_remote_url(repo_path: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(repo_path)
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

pub fn get_trunk_branch(repo_path: &Path) -> Option<String> {
    // try to get the default branch from origin
    let output = Command::new("git")
        .args(["symbolic-ref", "refs/remotes/origin/HEAD"])
        .current_dir(repo_path)
        .output()
        .ok()?;

    if output.status.success() {
        let full_ref = String::from_utf8_lossy(&output.stdout).trim().to_string();
        // refs/remotes/origin/main -> main
        full_ref
            .strip_prefix("refs/remotes/origin/")
            .map(|s| s.to_string())
    } else {
        // fallback to main or master
        for branch in ["main", "master"] {
            let check = Command::new("git")
                .args([
                    "rev-parse",
                    "--verify",
                    &format!("refs/remotes/origin/{branch}"),
                ])
                .current_dir(repo_path)
                .output()
                .ok()?;
            if check.status.success() {
                return Some(branch.to_string());
            }
        }
        None
    }
}

pub fn get_remote_ref(repo_path: &Path, branch: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", &format!("refs/remotes/origin/{branch}")])
        .current_dir(repo_path)
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

pub fn get_local_ref(repo_path: &Path, branch: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", &format!("refs/heads/{branch}")])
        .current_dir(repo_path)
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

pub fn count_commits(repo_path: &Path, old_sha: &str, new_sha: &str) -> Option<u64> {
    let output = Command::new("git")
        .args(["rev-list", "--count", &format!("{old_sha}..{new_sha}")])
        .current_dir(repo_path)
        .output()
        .ok()?;

    if output.status.success() {
        String::from_utf8_lossy(&output.stdout).trim().parse().ok()
    } else {
        None
    }
}

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// Returns all remote tracking branches and their SHAs.
/// e.g., {"main" => "abc123", "feature" => "def456"}
pub fn get_all_remote_refs(repo_path: &Path) -> HashMap<String, String> {
    let output = Command::new("git")
        .args([
            "for-each-ref",
            "--format=%(refname:short) %(objectname)",
            "refs/remotes/origin/",
        ])
        .current_dir(repo_path)
        .output()
        .ok();

    let mut refs = HashMap::new();
    if let Some(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                // format: "origin/main abc123..."
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    // strip "origin/" prefix
                    if let Some(branch) = parts[0].strip_prefix("origin/") {
                        // skip HEAD
                        if branch != "HEAD" {
                            refs.insert(branch.to_string(), parts[1].to_string());
                        }
                    }
                }
            }
        }
    }
    refs
}

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

/// List commits in range old_sha..new_sha (commits reachable from new but not old).
pub fn list_commits_in_range(repo_path: &Path, old_sha: &str, new_sha: &str) -> Vec<String> {
    let output = Command::new("git")
        .args(["rev-list", &format!("{}..{}", old_sha, new_sha)])
        .current_dir(repo_path)
        .output();

    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect(),
        _ => Vec::new(),
    }
}

/// List commits on the given branches that aren't reachable from any other remote branch.
/// Used for first-time pushes where we don't have an old SHA.
pub fn list_unique_commits(repo_path: &Path, branches: &[&str]) -> Vec<String> {
    // git rev-list origin/branch1 origin/branch2 ... --not --exclude=origin/branch1 --exclude=origin/branch2 ... --remotes=origin
    let mut args = vec!["rev-list".to_string()];

    // add all branches as positive refs
    for branch in branches {
        args.push(format!("refs/remotes/origin/{}", branch));
    }

    args.push("--not".to_string());

    // exclude all the branches we're including
    for branch in branches {
        args.push(format!("--exclude=origin/{}", branch));
    }

    args.push("--remotes=origin".to_string());

    let output = Command::new("git")
        .args(&args)
        .current_dir(repo_path)
        .output();

    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect(),
        _ => Vec::new(),
    }
}

/// Get the patch-id for a commit. Returns None if the commit has no diff (e.g., merge commits).
pub fn get_patch_id(repo_path: &Path, sha: &str) -> Option<String> {
    // git show <sha> | git patch-id --stable
    let show = Command::new("git")
        .args(["show", sha])
        .current_dir(repo_path)
        .output()
        .ok()?;

    if !show.status.success() {
        return None;
    }

    let patch_id = Command::new("git")
        .args(["patch-id", "--stable"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .current_dir(repo_path)
        .spawn()
        .ok()?;

    use std::io::Write;
    patch_id.stdin.as_ref()?.write_all(&show.stdout).ok()?;
    let output = patch_id.wait_with_output().ok()?;

    if output.status.success() {
        let line = String::from_utf8_lossy(&output.stdout);
        // format: "<patch-id> <commit-sha>"
        line.split_whitespace().next().map(|s| s.to_string())
    } else {
        None
    }
}

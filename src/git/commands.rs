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
    if let Some(output) = output
        && output.status.success() {
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

/// Check if a commit is reachable from any remote branch OTHER than the specified ones.
/// Used to filter out commits that came from fetch but are in the push range due to rebasing.
pub fn is_reachable_from_other_remote(
    repo_path: &Path,
    sha: &str,
    exclude_branches: &[&str],
) -> bool {
    // Get all remote branches except the ones we're pushing
    let refs_output = Command::new("git")
        .args([
            "for-each-ref",
            "--format=%(refname)",
            "refs/remotes/origin/",
        ])
        .current_dir(repo_path)
        .output();

    let other_refs: Vec<String> = match refs_output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter(|r| {
                let dominated = exclude_branches
                    .iter()
                    .any(|b| r.ends_with(&format!("/{}", b)));
                !dominated && !r.ends_with("/HEAD")
            })
            .map(|s| s.to_string())
            .collect(),
        _ => return false,
    };

    if other_refs.is_empty() {
        return false;
    }

    // Check if commit is reachable from any of these refs
    // git merge-base --is-ancestor <sha> <ref> returns 0 if sha is ancestor of ref
    for ref_name in &other_refs {
        let result = Command::new("git")
            .args(["merge-base", "--is-ancestor", sha, ref_name])
            .current_dir(repo_path)
            .output();

        if let Ok(o) = result
            && o.status.success() {
                return true;
            }
    }

    false
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

/// Returns total lines changed (added + removed) for a commit.
pub fn get_lines_changed(repo_path: &Path, sha: &str) -> Option<u64> {
    // git show --stat --format="" <sha>
    // outputs lines like: " file.rs | 10 ++++----"
    // with summary: " 3 files changed, 10 insertions(+), 5 deletions(-)"
    let output = Command::new("git")
        .args(["show", "--stat", "--format=", sha])
        .current_dir(repo_path)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // parse the last line for the summary
    let last_line = stdout.lines().last()?;

    let mut insertions = 0u64;
    let mut deletions = 0u64;

    // look for "N insertion" and "N deletion" patterns
    for word in last_line.split_whitespace().collect::<Vec<_>>().windows(2) {
        if word[1].starts_with("insertion") {
            insertions = word[0].parse().unwrap_or(0);
        } else if word[1].starts_with("deletion") {
            deletions = word[0].parse().unwrap_or(0);
        }
    }

    Some(insertions + deletions)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    struct TestRepo {
        path: std::path::PathBuf,
    }

    impl TestRepo {
        fn new() -> Self {
            let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
            let path =
                std::env::temp_dir().join(format!("party-test-{}-{}", std::process::id(), id));
            fs::create_dir_all(&path).unwrap();

            // init repo
            Command::new("git")
                .args(["init"])
                .current_dir(&path)
                .output()
                .unwrap();

            // configure user (required for commits)
            Command::new("git")
                .args(["config", "user.email", "test@test.com"])
                .current_dir(&path)
                .output()
                .unwrap();
            Command::new("git")
                .args(["config", "user.name", "Test"])
                .current_dir(&path)
                .output()
                .unwrap();

            Self { path }
        }

        fn write_file(&self, name: &str, content: &str) {
            fs::write(self.path.join(name), content).unwrap();
        }

        fn commit(&self, msg: &str) -> String {
            Command::new("git")
                .args(["add", "."])
                .current_dir(&self.path)
                .output()
                .unwrap();

            Command::new("git")
                .args(["commit", "-m", msg])
                .current_dir(&self.path)
                .output()
                .unwrap();

            // get the sha
            let output = Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(&self.path)
                .output()
                .unwrap();

            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
    }

    impl Drop for TestRepo {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn lines_changed_counts_insertions() {
        let repo = TestRepo::new();
        repo.write_file("test.txt", "line1\nline2\nline3\n");
        let sha = repo.commit("add 3 lines");

        let lines = get_lines_changed(&repo.path, &sha);
        assert_eq!(lines, Some(3));
    }

    #[test]
    fn lines_changed_counts_deletions() {
        let repo = TestRepo::new();
        repo.write_file("test.txt", "line1\nline2\nline3\n");
        repo.commit("initial");

        repo.write_file("test.txt", "line1\n");
        let sha = repo.commit("delete 2 lines");

        let lines = get_lines_changed(&repo.path, &sha);
        assert_eq!(lines, Some(2));
    }

    #[test]
    fn lines_changed_counts_both() {
        let repo = TestRepo::new();
        repo.write_file("test.txt", "aaa\nbbb\nccc\n");
        repo.commit("initial");

        repo.write_file("test.txt", "aaa\nBBB\nccc\nddd\n");
        let sha = repo.commit("modify and add");

        // 1 deletion (bbb) + 2 insertions (BBB, ddd) = 3
        let lines = get_lines_changed(&repo.path, &sha);
        assert_eq!(lines, Some(3));
    }

    #[test]
    fn lines_changed_single_line_addition() {
        let repo = TestRepo::new();
        repo.write_file("test.txt", "line1\nline2\n");
        repo.commit("initial");

        repo.write_file("test.txt", "line1\nline2\nline3\n");
        let sha = repo.commit("add one line");

        let lines = get_lines_changed(&repo.path, &sha);
        assert_eq!(lines, Some(1));
    }

    #[test]
    fn lines_changed_invalid_sha_returns_none() {
        let repo = TestRepo::new();
        repo.write_file("test.txt", "content\n");
        repo.commit("initial");

        let lines = get_lines_changed(&repo.path, "invalid-sha");
        assert_eq!(lines, None);
    }
}

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

/// Every file git currently tracks — the universe `discover.rs` scans for
/// project-marker files.
pub fn ls_files(repo_dir: &Path) -> Result<Vec<String>> {
    run_git_lines(repo_dir, &["ls-files"])
}

/// Files that differ between `range` (any `git diff` range expression —
/// `main..HEAD`, `main...HEAD` for merge-base semantics, a single commit,
/// or `HEAD` alone for working-tree changes) and the current tree.
pub fn changed_files(repo_dir: &Path, range: &str) -> Result<Vec<String>> {
    run_git_lines(repo_dir, &["diff", "--name-only", range])
}

fn run_git_lines(repo_dir: &Path, args: &[&str]) -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_dir)
        .output()
        .with_context(|| format!("running git {args:?}"))?;
    if !output.status.success() {
        anyhow::bail!(
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_repo(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("affected-test-git-{}-{}", std::process::id(), name));
        fs::create_dir_all(&dir).unwrap();
        let run = |args: &[&str]| {
            let status = Command::new("git")
                .args(args)
                .current_dir(&dir)
                .status()
                .unwrap();
            assert!(status.success(), "git {args:?} failed");
        };
        run(&["init", "-q"]);
        run(&["config", "user.email", "test@example.com"]);
        run(&["config", "user.name", "test"]);
        dir
    }

    fn commit_all(dir: &std::path::Path, message: &str) {
        Command::new("git")
            .args(["add", "-A"])
            .current_dir(dir)
            .status()
            .unwrap();
        Command::new("git")
            .args(["commit", "-q", "-m", message])
            .current_dir(dir)
            .status()
            .unwrap();
    }

    #[test]
    fn ls_files_lists_every_tracked_file() {
        let repo = temp_repo("lsfiles");
        fs::create_dir_all(repo.join("svc")).unwrap();
        fs::write(repo.join("svc/Cargo.toml"), "[package]\nname=\"svc\"").unwrap();
        fs::write(repo.join("README.md"), "hi").unwrap();
        commit_all(&repo, "initial");

        let files = ls_files(&repo).unwrap();
        assert!(files.contains(&"svc/Cargo.toml".to_string()));
        assert!(files.contains(&"README.md".to_string()));

        fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn changed_files_reflects_a_real_diff_between_two_commits() {
        let repo = temp_repo("diff");
        fs::create_dir_all(repo.join("svc")).unwrap();
        fs::write(repo.join("svc/Cargo.toml"), "[package]\nname=\"svc\"").unwrap();
        fs::write(repo.join("README.md"), "v1").unwrap();
        commit_all(&repo, "initial");
        let first = String::from_utf8(
            Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(&repo)
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap()
        .trim()
        .to_string();

        fs::write(repo.join("svc/src.rs"), "fn main() {}").unwrap();
        fs::create_dir_all(repo.join("svc/src")).unwrap();
        commit_all(&repo, "add source file");

        let changed = changed_files(&repo, &format!("{first}..HEAD")).unwrap();
        assert_eq!(changed, vec!["svc/src.rs".to_string()]);
        assert!(
            !changed.contains(&"README.md".to_string()),
            "unchanged file must not appear in the diff"
        );

        fs::remove_dir_all(&repo).ok();
    }
}

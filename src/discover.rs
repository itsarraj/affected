use std::path::Path;

/// Files whose *presence* marks a directory as a project root — the same
/// signal `dockergen` uses to detect a stack elsewhere in this workspace,
/// reused here for a different purpose: not "what stack is this," just
/// "is this a project boundary."
const MARKER_FILES: &[&str] = &["Cargo.toml", "package.json", "pyproject.toml", "go.mod"];

/// Finds every project root in a repo, given its **full tracked file
/// list** (from `git ls-files` — see `git.rs`) rather than walking the
/// filesystem directly, so this stays pure and testable with a plain
/// `Vec<String>` fixture. A directory is a project root if it directly
/// contains one of `MARKER_FILES`; `"."` represents a marker file sitting
/// at the repo's own top level.
pub fn find_project_roots(all_files: &[String]) -> Vec<String> {
    let mut roots: Vec<String> = all_files
        .iter()
        .filter_map(|f| {
            let path = Path::new(f);
            let name = path.file_name()?.to_str()?;
            if !MARKER_FILES.contains(&name) {
                return None;
            }
            let parent = path.parent()?.to_str()?;
            Some(if parent.is_empty() {
                ".".to_string()
            } else {
                parent.to_string()
            })
        })
        .collect();
    roots.sort();
    roots.dedup();
    roots
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_one_root_per_marker_file() {
        let files = vec![
            "apps/pgqueue/Cargo.toml".to_string(),
            "apps/pgqueue/src/main.rs".to_string(),
            "apps/web/package.json".to_string(),
            "notes/journal.md".to_string(),
        ];
        let roots = find_project_roots(&files);
        assert_eq!(
            roots,
            vec!["apps/pgqueue".to_string(), "apps/web".to_string()]
        );
    }

    #[test]
    fn a_marker_file_at_the_repo_root_becomes_the_dot_root() {
        let files = vec!["Cargo.toml".to_string(), "src/main.rs".to_string()];
        assert_eq!(find_project_roots(&files), vec![".".to_string()]);
    }

    #[test]
    fn duplicate_marker_files_in_the_same_dir_collapse_to_one_root() {
        // Shouldn't happen in practice (one dir, two marker files), but
        // must not produce a duplicate root if it does.
        let files = vec!["svc/Cargo.toml".to_string(), "svc/package.json".to_string()];
        assert_eq!(find_project_roots(&files), vec!["svc".to_string()]);
    }

    #[test]
    fn no_marker_files_anywhere_finds_nothing() {
        let files = vec!["README.md".to_string(), "notes/a.md".to_string()];
        assert!(find_project_roots(&files).is_empty());
    }

    #[test]
    fn results_are_sorted() {
        let files = vec!["z/Cargo.toml".to_string(), "a/Cargo.toml".to_string()];
        assert_eq!(
            find_project_roots(&files),
            vec!["a".to_string(), "z".to_string()]
        );
    }
}

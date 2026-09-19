use std::collections::BTreeSet;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AffectedResult {
    pub projects: BTreeSet<String>,
    /// Changed files that don't fall under any known project root (a
    /// top-level README, a `notes/` file, ...) — reported explicitly
    /// rather than silently dropped, since "nothing matched" and
    /// "everything matched cleanly" are different, both useful facts.
    pub unmatched: Vec<String>,
}

/// Maps each changed file to the **most specific** (longest-path)
/// project root that contains it — so a change to `apps/web/src/x.ts`
/// with both `apps/web` and a hypothetical outer `apps` root registered
/// resolves to `apps/web`, not the broader one. Pure: takes the already-
/// discovered root list rather than rediscovering it, so this is testable
/// with plain fixtures independent of `discover.rs`/`git.rs`.
pub fn map_changed_to_projects(changed_files: &[String], roots: &[String]) -> AffectedResult {
    let mut result = AffectedResult::default();
    for file in changed_files {
        match roots
            .iter()
            .filter(|r| is_under(file, r))
            .max_by_key(|r| r.len())
        {
            Some(root) => {
                result.projects.insert(root.clone());
            }
            None => result.unmatched.push(file.clone()),
        }
    }
    result
}

fn is_under(file: &str, root: &str) -> bool {
    if root == "." {
        return true;
    }
    file.starts_with(root) && file.as_bytes().get(root.len()) == Some(&b'/')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roots(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }
    fn files(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn maps_a_changed_file_to_its_containing_project() {
        let result = map_changed_to_projects(
            &files(&["tools/pgqueue/src/main.rs"]),
            &roots(&["tools/pgqueue", "tools/leakscan"]),
        );
        assert_eq!(
            result.projects,
            ["tools/pgqueue".to_string()].into_iter().collect()
        );
        assert!(result.unmatched.is_empty());
    }

    #[test]
    fn most_specific_nested_root_wins_over_a_broader_one() {
        let result = map_changed_to_projects(
            &files(&["apps/web/src/index.ts"]),
            &roots(&["apps", "apps/web"]),
        );
        assert_eq!(
            result.projects,
            ["apps/web".to_string()].into_iter().collect()
        );
    }

    #[test]
    fn a_file_outside_every_known_project_is_unmatched_not_dropped() {
        let result = map_changed_to_projects(
            &files(&["README.md", "notes/journal.md"]),
            &roots(&["tools/pgqueue"]),
        );
        assert!(result.projects.is_empty());
        assert_eq!(
            result.unmatched,
            vec!["README.md".to_string(), "notes/journal.md".to_string()]
        );
    }

    #[test]
    fn multiple_changed_files_in_the_same_project_produce_one_entry_not_a_duplicate() {
        let result = map_changed_to_projects(
            &files(&[
                "tools/pgqueue/src/main.rs",
                "tools/pgqueue/src/lib.rs",
                "tools/pgqueue/Cargo.toml",
            ]),
            &roots(&["tools/pgqueue"]),
        );
        assert_eq!(result.projects.len(), 1);
    }

    #[test]
    fn a_file_named_like_a_project_dir_but_not_inside_it_does_not_falsely_match() {
        // "tools/pgqueue-extra/x.rs" must not match root "tools/pgqueue" —
        // a naive `starts_with` without the trailing-slash check would.
        let result = map_changed_to_projects(
            &files(&["tools/pgqueue-extra/x.rs"]),
            &roots(&["tools/pgqueue"]),
        );
        assert!(result.projects.is_empty());
        assert_eq!(
            result.unmatched,
            vec!["tools/pgqueue-extra/x.rs".to_string()]
        );
    }
}

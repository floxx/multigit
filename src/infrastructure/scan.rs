//! Finds repositories below a folder picked with (+).

use std::path::{Path, PathBuf};

/// A folder is a repository when it contains `.git` (a directory, or a file for
/// worktrees and submodules).
pub fn is_repo(dir: &Path) -> bool {
    dir.join(".git").exists()
}

/// `root` itself if it is a repository; otherwise every repository at most `depth`
/// levels below it. Does not descend into repositories it found. Results are
/// canonical paths, sorted.
#[tracing::instrument(skip_all, fields(root = %root.display(), depth))]
pub fn scan(root: &Path, depth: usize) -> Vec<PathBuf> {
    let mut found = Vec::new();
    if is_repo(root) {
        found.push(root.to_path_buf());
    } else {
        walk(root, depth, &mut found);
    }
    let mut found: Vec<PathBuf> = found
        .into_iter()
        .map(|p| p.canonicalize().unwrap_or(p))
        .collect();
    found.sort();
    found.dedup();
    found
}

fn walk(dir: &Path, depth: usize, found: &mut Vec<PathBuf>) {
    if depth == 0 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if is_repo(&path) {
            found.push(path);
        } else {
            walk(&path, depth - 1, found);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn mkrepo(path: &Path) {
        fs::create_dir_all(path.join(".git")).unwrap();
    }

    #[test]
    fn root_repo_is_returned_alone() {
        let dir = tempfile::tempdir().unwrap();
        mkrepo(dir.path());
        mkrepo(&dir.path().join("nested"));
        assert_eq!(
            scan(dir.path(), 2),
            vec![dir.path().canonicalize().unwrap()]
        );
    }

    #[test]
    fn depth_limits_and_no_descent_into_repos() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        mkrepo(&root.join("a"));
        mkrepo(&root.join("a/inner"));
        mkrepo(&root.join("org/b"));
        mkrepo(&root.join("deep/x/c"));
        fs::write(root.join("file.txt"), "x").unwrap();
        let canon = |p: &str| root.join(p).canonicalize().unwrap();

        assert_eq!(scan(root, 2), vec![canon("a"), canon("org/b")]);
        assert_eq!(scan(root, 1), vec![canon("a")]);
        assert_eq!(
            scan(root, 3),
            vec![canon("a"), canon("deep/x/c"), canon("org/b")]
        );
        assert!(scan(root, 0).is_empty());
    }

    #[test]
    fn git_file_counts_as_repo() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("wt")).unwrap();
        fs::write(dir.path().join("wt/.git"), "gitdir: /elsewhere").unwrap();
        assert_eq!(scan(dir.path(), 1).len(), 1);
    }

    #[test]
    fn missing_root_gives_nothing() {
        assert!(scan(Path::new("/does/not/exist"), 2).is_empty());
    }
}

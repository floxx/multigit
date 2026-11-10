//! Helpers for tests that need real git repositories in temporary directories.
//! Every git call is isolated from the user's global and system git config.

use crate::infrastructure::git::GitRunner;
use std::path::{Path, PathBuf};
use std::process::Command;

const ISOLATION_ENV: &[(&str, &str)] = &[
    ("GIT_CONFIG_GLOBAL", "/dev/null"),
    ("GIT_CONFIG_NOSYSTEM", "1"),
    ("GIT_AUTHOR_NAME", "Test"),
    ("GIT_AUTHOR_EMAIL", "test@example.invalid"),
    ("GIT_COMMITTER_NAME", "Test"),
    ("GIT_COMMITTER_EMAIL", "test@example.invalid"),
    ("GIT_CONFIG_COUNT", "1"),
    ("GIT_CONFIG_KEY_0", "init.defaultBranch"),
    ("GIT_CONFIG_VALUE_0", "main"),
];

/// A runner that ignores the user's git configuration.
pub fn isolated_runner() -> GitRunner {
    ISOLATION_ENV
        .iter()
        .fold(GitRunner::default(), |r, (k, v)| r.with_env(*k, *v))
}

/// A repository in a temporary directory, deleted on drop.
pub struct TestRepo {
    _dir: tempfile::TempDir,
    path: PathBuf,
}

impl Default for TestRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl TestRepo {
    /// `git init` in a fresh temporary directory.
    pub fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("repo");
        std::fs::create_dir(&path).expect("mkdir");
        let repo = Self { _dir: dir, path };
        repo.git(&["init", "-q"]);
        repo
    }

    /// A bare repository, for use as a remote.
    pub fn bare() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("remote.git");
        let repo = Self { _dir: dir, path };
        git_in(repo._dir.path(), &["init", "-q", "--bare", "remote.git"]);
        repo
    }

    /// Clone `remote` into a fresh temporary directory.
    pub fn clone_of(remote: &TestRepo) -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("clone");
        let url = remote.path().to_string_lossy().into_owned();
        git_in(dir.path(), &["clone", "-q", &url, "clone"]);
        Self { _dir: dir, path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Run git in the repo; panics on failure. Returns stdout.
    pub fn git(&self, args: &[&str]) -> String {
        git_in(&self.path, args)
    }

    pub fn write(&self, file: &str, content: &str) {
        std::fs::write(self.path.join(file), content).expect("write");
    }

    /// Write a file, stage it and commit it.
    pub fn commit(&self, file: &str, content: &str) {
        self.write(file, content);
        self.git(&["add", file]);
        self.git(&["commit", "-q", "-m", file]);
    }
}

fn git_in(dir: &Path, args: &[&str]) -> String {
    let mut cmd = Command::new("git");
    cmd.current_dir(dir).args(args);
    for (k, v) in ISOLATION_ENV {
        cmd.env(k, v);
    }
    let out = cmd.output().expect("spawn git");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

//! Hand-written descriptions and value hints for the most used git commands.
//!
//! git's own `--git-completion-helper` lists option names but no descriptions, and
//! for revision/diff commands (`log`, `diff`, `show`, …) it omits most options.
//! This table fills those gaps for the commands used daily.

/// What kind of value a positional argument of a command takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    /// Local branches.
    Branches,
    /// Local and remote branches.
    BranchesAndRemote,
    /// Local branches, remote branches and tags.
    Refs,
    /// Configured remotes.
    Remotes,
    /// First positional: remote; later: local branches.
    RemoteThenBranch,
    /// Tags.
    Tags,
    /// Changed, untracked or conflicted paths.
    Files,
    /// First positional: one of the listed subcommands; later: the given kind.
    Subcommands(&'static [(&'static str, &'static str)], &'static ValueKind),
    /// No value completion.
    Nothing,
}

/// Description, common options and positional value kind of one command.
#[derive(Debug, Clone, Copy)]
pub struct CommandHint {
    pub name: &'static str,
    pub description: &'static str,
    pub options: &'static [(&'static str, &'static str)],
    pub values: ValueKind,
}

/// Look up the hint for a subcommand.
pub fn hint(name: &str) -> Option<&'static CommandHint> {
    HINTS.iter().find(|h| h.name == name)
}

const REMOTE_SUBCOMMANDS: &[(&str, &str)] = &[
    ("add", "Add a remote"),
    ("get-url", "Show the URL of a remote"),
    ("prune", "Delete stale remote-tracking branches"),
    ("remove", "Remove a remote"),
    ("rename", "Rename a remote"),
    ("set-head", "Set the default branch of a remote"),
    ("set-url", "Change the URL of a remote"),
    ("show", "Show information about a remote"),
];

const STASH_SUBCOMMANDS: &[(&str, &str)] = &[
    ("apply", "Apply a stash without removing it"),
    ("branch", "Create a branch from a stash"),
    ("clear", "Remove all stashes"),
    ("drop", "Remove one stash"),
    ("list", "List stashes"),
    ("pop", "Apply a stash and remove it"),
    ("push", "Save local changes to a new stash"),
    ("show", "Show the changes in a stash"),
];

/// The hint table.
pub const HINTS: &[CommandHint] = &[
    CommandHint {
        name: "add",
        description: "Add file contents to the index",
        options: &[
            ("--all", "Stage all changes, including deletions"),
            ("--dry-run", "Only show what would be added"),
            ("--force", "Also add ignored files"),
            ("--update", "Stage changes of tracked files only"),
            ("--verbose", "Be verbose"),
            ("-A", "Stage all changes, including deletions"),
            ("-u", "Stage changes of tracked files only"),
        ],
        values: ValueKind::Files,
    },
    CommandHint {
        name: "branch",
        description: "List, create, or delete branches",
        options: &[
            ("--all", "List local and remote branches"),
            ("--merged", "Only branches merged into HEAD"),
            ("--no-merged", "Only branches not merged into HEAD"),
            ("--remotes", "List remote branches"),
            ("--set-upstream-to=", "Set the upstream of a branch"),
            ("--show-current", "Print the current branch name"),
            ("--unset-upstream", "Remove the upstream of a branch"),
            ("-D", "Delete a branch even if not merged"),
            ("-a", "List local and remote branches"),
            ("-d", "Delete a merged branch"),
            ("-m", "Rename a branch"),
            ("-r", "List remote branches"),
            ("-v", "Show last commit per branch"),
            ("-vv", "Show upstream and last commit per branch"),
        ],
        values: ValueKind::Branches,
    },
    CommandHint {
        name: "checkout",
        description: "Switch branches or restore working tree files",
        options: &[
            ("--", "Following arguments are paths"),
            ("--detach", "Detach HEAD at the given commit"),
            ("--force", "Throw away local changes"),
            ("--track", "Set up tracking of the remote branch"),
            ("-B", "Create or reset a branch and switch to it"),
            ("-b", "Create a branch and switch to it"),
            ("-f", "Throw away local changes"),
        ],
        values: ValueKind::Refs,
    },
    CommandHint {
        name: "cherry-pick",
        description: "Apply the changes introduced by existing commits",
        options: &[
            ("--abort", "Cancel and return to the previous state"),
            ("--continue", "Continue after resolving conflicts"),
            ("--no-commit", "Apply without committing"),
            ("--skip", "Skip the current commit"),
            ("-x", "Append \"cherry picked from\" to the message"),
        ],
        values: ValueKind::Refs,
    },
    CommandHint {
        name: "clean",
        description: "Remove untracked files from the working tree",
        options: &[
            ("--dry-run", "Only show what would be removed"),
            ("--force", "Really remove files"),
            ("-X", "Remove only ignored files"),
            ("-d", "Also remove untracked directories"),
            ("-f", "Really remove files"),
            ("-n", "Only show what would be removed"),
            ("-x", "Also remove ignored files"),
        ],
        values: ValueKind::Files,
    },
    CommandHint {
        name: "commit",
        description: "Record changes to the repository (needs -m here)",
        options: &[
            ("--all", "Commit all changes of tracked files"),
            ("--allow-empty", "Allow a commit without changes"),
            ("--amend", "Replace the last commit"),
            ("--fixup=", "Create a fixup commit for the given commit"),
            ("--message=", "Commit message"),
            ("--no-edit", "Keep the existing message (with --amend)"),
            ("--no-verify", "Skip pre-commit and commit-msg hooks"),
            ("--signoff", "Add a Signed-off-by trailer"),
            ("-a", "Commit all changes of tracked files"),
            ("-m", "Commit message"),
        ],
        values: ValueKind::Files,
    },
    CommandHint {
        name: "diff",
        description: "Show changes between commits, index and working tree",
        options: &[
            ("--cached", "Compare the index with HEAD"),
            ("--check", "Warn about whitespace errors"),
            ("--color-words", "Show a word-level diff"),
            ("--name-only", "Show only names of changed files"),
            ("--name-status", "Show names and status of changed files"),
            ("--staged", "Compare the index with HEAD"),
            ("--stat", "Show a diffstat"),
            ("--word-diff", "Show a word-level diff"),
            ("-w", "Ignore whitespace"),
        ],
        values: ValueKind::Refs,
    },
    CommandHint {
        name: "fetch",
        description: "Download objects and refs from another repository",
        options: &[
            ("--all", "Fetch all remotes"),
            ("--depth=", "Limit fetching to the given depth"),
            ("--dry-run", "Only show what would be fetched"),
            (
                "--prune",
                "Remove remote-tracking refs that no longer exist",
            ),
            ("--tags", "Fetch all tags"),
            ("--unshallow", "Convert a shallow clone to a complete one"),
            ("-p", "Remove remote-tracking refs that no longer exist"),
        ],
        values: ValueKind::RemoteThenBranch,
    },
    CommandHint {
        name: "log",
        description: "Show commit logs",
        options: &[
            ("--all", "Show commits of all refs"),
            ("--author=", "Only commits by this author"),
            ("--decorate", "Show ref names"),
            ("--first-parent", "Follow only the first parent of merges"),
            ("--format=", "Pretty-print with the given format"),
            ("--graph", "Draw the commit graph"),
            ("--grep=", "Only commits whose message matches"),
            ("--max-count=", "Limit the number of commits"),
            ("--merges", "Only merge commits"),
            ("--name-only", "Show names of changed files"),
            ("--no-merges", "Skip merge commits"),
            ("--oneline", "One line per commit"),
            ("--patch", "Show the diff of each commit"),
            ("--reverse", "Oldest commits first"),
            ("--since=", "Only commits newer than a date"),
            ("--stat", "Show a diffstat per commit"),
            ("--until=", "Only commits older than a date"),
            ("-n", "Limit the number of commits"),
            ("-p", "Show the diff of each commit"),
        ],
        values: ValueKind::Refs,
    },
    CommandHint {
        name: "merge",
        description: "Join two or more development histories together",
        options: &[
            ("--abort", "Cancel the merge"),
            ("--continue", "Continue after resolving conflicts"),
            ("--ff-only", "Only fast-forward"),
            ("--no-edit", "Accept the generated message"),
            ("--no-ff", "Always create a merge commit"),
            (
                "--squash",
                "Squash into the working tree without committing",
            ),
            ("-m", "Merge commit message"),
        ],
        values: ValueKind::BranchesAndRemote,
    },
    CommandHint {
        name: "pull",
        description: "Fetch from and integrate with another repository",
        options: &[
            ("--all", "Fetch all remotes"),
            (
                "--autostash",
                "Stash local changes before and restore after",
            ),
            ("--ff-only", "Only fast-forward"),
            ("--no-rebase", "Merge instead of rebasing"),
            (
                "--prune",
                "Remove remote-tracking refs that no longer exist",
            ),
            ("--rebase", "Rebase local commits on top of upstream"),
            ("--tags", "Fetch all tags"),
            ("--verbose", "Be verbose"),
        ],
        values: ValueKind::RemoteThenBranch,
    },
    CommandHint {
        name: "push",
        description: "Update remote refs along with associated objects",
        options: &[
            ("--all", "Push all branches"),
            ("--delete", "Delete the remote branch"),
            ("--dry-run", "Only show what would be pushed"),
            (
                "--follow-tags",
                "Also push annotated tags reachable from pushed refs",
            ),
            ("--force", "Overwrite the remote branch"),
            (
                "--force-with-lease",
                "Overwrite only if remote is as expected",
            ),
            ("--no-verify", "Skip the pre-push hook"),
            ("--set-upstream", "Set the pushed branch as upstream"),
            ("--tags", "Push all tags"),
            ("-f", "Overwrite the remote branch"),
            ("-u", "Set the pushed branch as upstream"),
        ],
        values: ValueKind::RemoteThenBranch,
    },
    CommandHint {
        name: "rebase",
        description: "Reapply commits on top of another base tip",
        options: &[
            ("--abort", "Cancel the rebase"),
            ("--autosquash", "Apply fixup!/squash! commits"),
            (
                "--autostash",
                "Stash local changes before and restore after",
            ),
            ("--continue", "Continue after resolving conflicts"),
            ("--onto", "Rebase onto the given base"),
            ("--skip", "Skip the current commit"),
        ],
        values: ValueKind::BranchesAndRemote,
    },
    CommandHint {
        name: "remote",
        description: "Manage the set of tracked repositories",
        options: &[("-v", "Show URLs")],
        values: ValueKind::Subcommands(REMOTE_SUBCOMMANDS, &ValueKind::Remotes),
    },
    CommandHint {
        name: "reset",
        description: "Reset current HEAD to the specified state",
        options: &[
            ("--hard", "Reset index and working tree (discards changes)"),
            ("--keep", "Reset, but keep local changes"),
            ("--merge", "Reset, keeping unstaged changes"),
            ("--mixed", "Reset the index, keep the working tree"),
            ("--soft", "Move HEAD only"),
        ],
        values: ValueKind::Refs,
    },
    CommandHint {
        name: "restore",
        description: "Restore working tree files",
        options: &[
            ("--source=", "Restore from the given commit"),
            ("--staged", "Restore the index (unstage)"),
            ("--worktree", "Restore the working tree"),
        ],
        values: ValueKind::Files,
    },
    CommandHint {
        name: "show",
        description: "Show various types of objects",
        options: &[
            ("--format=", "Pretty-print with the given format"),
            ("--name-only", "Show names of changed files"),
            ("--oneline", "One line for the commit"),
            ("--stat", "Show a diffstat"),
        ],
        values: ValueKind::Refs,
    },
    CommandHint {
        name: "stash",
        description: "Stash the changes in a dirty working directory away",
        options: &[
            ("--include-untracked", "Also stash untracked files"),
            ("--keep-index", "Keep staged changes in the index"),
            ("-m", "Stash message"),
            ("-u", "Also stash untracked files"),
        ],
        values: ValueKind::Subcommands(STASH_SUBCOMMANDS, &ValueKind::Nothing),
    },
    CommandHint {
        name: "status",
        description: "Show the working tree status",
        options: &[
            ("--branch", "Show branch information"),
            ("--ignored", "Also show ignored files"),
            ("--short", "Short format"),
            (
                "--untracked-files=",
                "Show untracked files (no, normal, all)",
            ),
            ("-b", "Show branch information"),
            ("-s", "Short format"),
            ("-sb", "Short format with branch information"),
        ],
        values: ValueKind::Files,
    },
    CommandHint {
        name: "switch",
        description: "Switch branches",
        options: &[
            ("-", "Switch to the previous branch"),
            ("--create", "Create a branch and switch to it"),
            ("--detach", "Switch to a commit with detached HEAD"),
            ("--discard-changes", "Throw away local changes"),
            ("--track", "Set up tracking of the remote branch"),
            ("-C", "Create or reset a branch and switch to it"),
            ("-c", "Create a branch and switch to it"),
        ],
        values: ValueKind::BranchesAndRemote,
    },
    CommandHint {
        name: "tag",
        description: "Create, list, delete or verify tags",
        options: &[
            ("--contains", "Only tags containing the commit"),
            ("--list", "List tags"),
            ("--sort=", "Sort by the given key"),
            ("-a", "Create an annotated tag"),
            ("-d", "Delete a tag"),
            ("-l", "List tags"),
            ("-m", "Tag message"),
        ],
        values: ValueKind::Tags,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_least_twenty_commands_unique() {
        assert!(HINTS.len() >= 20);
        let mut names: Vec<&str> = HINTS.iter().map(|h| h.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), HINTS.len());
    }

    #[test]
    fn options_are_sorted_within_each_command() {
        for h in HINTS {
            let names: Vec<&str> = h.options.iter().map(|o| o.0).collect();
            let mut sorted = names.clone();
            sorted.sort_unstable();
            assert_eq!(names, sorted, "options of {} not sorted", h.name);
        }
    }

    #[test]
    fn lookup() {
        assert_eq!(hint("pull").map(|h| h.name), Some("pull"));
        assert!(hint("does-not-exist").is_none());
    }
}

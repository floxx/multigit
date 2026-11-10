//! Detection of commands that can destroy work and need a confirmation for bulk runs.

/// Global git options that take a separate value argument (`-C <path>`, `-c <k=v>`).
const GLOBAL_OPTIONS_WITH_VALUE: &[&str] = &["-C", "-c", "--git-dir", "--work-tree", "--namespace"];

/// Split args into the subcommand and its own arguments, skipping global options.
pub fn split_subcommand(args: &[String]) -> Option<(&str, &[String])> {
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        if GLOBAL_OPTIONS_WITH_VALUE.contains(&arg) {
            i += 2;
        } else if arg.starts_with('-') {
            i += 1;
        } else {
            return Some((arg, &args[i + 1..]));
        }
    }
    None
}

fn has(rest: &[String], long: &str) -> bool {
    rest.iter()
        .any(|a| a == long || a.starts_with(&format!("{long}=")))
}

/// True when a short-option cluster such as `-fdx` contains `flag`.
fn has_short(rest: &[String], flag: char) -> bool {
    rest.iter()
        .take_while(|a| a.as_str() != "--")
        .any(|a| a.starts_with('-') && !a.starts_with("--") && a[1..].contains(flag))
}

/// Whether running `git <args>` may discard work that cannot be recovered easily.
///
/// Covered: `reset --hard`, `push -f/--force*`, `clean -f`, `branch -D`,
/// `checkout -- <paths>` / `checkout -f`, any `restore`, `stash drop` / `stash clear`.
pub fn is_destructive(args: &[String]) -> bool {
    let Some((cmd, rest)) = split_subcommand(args) else {
        return false;
    };
    match cmd {
        "reset" => has(rest, "--hard"),
        "push" => {
            has(rest, "--force")
                || has(rest, "--force-with-lease")
                || has(rest, "--force-if-includes")
                || has_short(rest, 'f')
        }
        "clean" => has(rest, "--force") || has_short(rest, 'f'),
        "branch" => {
            has_short(rest, 'D')
                || ((has(rest, "--delete") || has_short(rest, 'd'))
                    && (has(rest, "--force") || has_short(rest, 'f')))
        }
        "checkout" => {
            rest.iter().any(|a| a == "--") || has(rest, "--force") || has_short(rest, 'f')
        }
        "restore" => true,
        "stash" => rest
            .iter()
            .find(|a| !a.starts_with('-'))
            .is_some_and(|sub| sub == "drop" || sub == "clear"),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(line: &str) -> bool {
        let args: Vec<String> = line.split_whitespace().map(String::from).collect();
        is_destructive(&args)
    }

    #[test]
    fn destructive_commands() {
        for line in [
            "reset --hard",
            "reset --hard HEAD~1",
            "push -f",
            "push -uf origin main",
            "push --force",
            "push --force-with-lease",
            "push --force-with-lease=main:abc",
            "clean -f",
            "clean -fdx",
            "clean --force -d",
            "branch -D topic",
            "branch -d -f topic",
            "branch --delete --force topic",
            "checkout -- .",
            "checkout -f main",
            "restore file.txt",
            "restore --staged file.txt",
            "stash drop",
            "stash clear",
            "-C sub reset --hard",
            "--no-pager reset --hard",
        ] {
            assert!(d(line), "{line} should be destructive");
        }
    }

    #[test]
    fn safe_commands() {
        for line in [
            "",
            "status",
            "reset",
            "reset --soft HEAD~1",
            "push",
            "push -u origin main",
            "pull --ff-only",
            "clean -n",
            "branch -d topic",
            "branch -a",
            "checkout main",
            "stash",
            "stash pop",
            "stash list",
            "fetch --all --prune",
            "-c core.x=reset --version",
        ] {
            assert!(!d(line), "{line} should be safe");
        }
    }
}

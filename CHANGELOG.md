# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.1.0]

### Added

- Board of repository cards showing branch, staged/modified/untracked/conflict counts, rebase or merge state, and unpushed/unpulled commits.
- Status refresh of all cards every 10 seconds and a background `git fetch --all --prune` every minute (configurable).
- (+) adds a repository, or every repository up to two folder levels below a chosen folder; × and "Remove marked" take cards off the board without touching files.
- Card buttons, by default `pull --ff-only`, `pull --rebase`, `push` and `fetch --all --prune`, editable in Settings with presets.
- Marking cards (individually, or all / none / dirty / behind) and running toolbar buttons on all marked repositories.
- Command line for any git command on the marked repositories (a typed leading `git` is ignored), with Tab completion of subcommands, options, branches, remotes, tags and files, and a clickable suggestion popup.
- Command history with inline suggestions, filtered Up/Down and Ctrl+R fuzzy search, kept across restarts.
- Output panel with one section per repository and exit code for the last 50 commands.
- Confirmation before destructive commands (such as `reset --hard` or `push --force`) run in more than one repository.
- git runs without prompts, pager or editor; commands that need an editor fail with a clear message and failed logins show as "auth failed" on the card.
- Application menu entry and icon, registered on every start and by the release tarball's `install.sh`.

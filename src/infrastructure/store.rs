//! Config and history files on disk (XDG locations).

use crate::domain::config::Config;
use crate::domain::history::History;
use std::io;
use std::path::{Path, PathBuf};

/// Where the app keeps its files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paths {
    pub config_file: PathBuf,
    pub history_file: PathBuf,
}

impl Paths {
    /// `$XDG_CONFIG_HOME/multigit/config.toml` and `$XDG_STATE_HOME/multigit/history`,
    /// falling back to `~/.config` and `~/.local/state`.
    pub fn from_env() -> Option<Self> {
        let var = |k: &str| {
            std::env::var_os(k)
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
        };
        let home = var("HOME");
        let config = var("XDG_CONFIG_HOME").or_else(|| home.as_ref().map(|h| h.join(".config")))?;
        let state =
            var("XDG_STATE_HOME").or_else(|| home.as_ref().map(|h| h.join(".local/state")))?;
        Some(Self::in_dirs(&config, &state))
    }

    pub fn in_dirs(config_dir: &Path, state_dir: &Path) -> Self {
        Self {
            config_file: config_dir.join("multigit/config.toml"),
            history_file: state_dir.join("multigit/history"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("{path}: {source}")]
    Io { path: PathBuf, source: io::Error },
    #[error("{path}: {source}")]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("cannot serialize config: {0}")]
    Serialize(#[from] toml::ser::Error),
}

fn io_err(path: &Path) -> impl FnOnce(io::Error) -> StoreError + '_ {
    move |source| StoreError::Io {
        path: path.to_path_buf(),
        source,
    }
}

/// Load the config; a missing file gives the defaults.
pub fn load_config(path: &Path) -> Result<Config, StoreError> {
    match std::fs::read_to_string(path) {
        Ok(text) => Config::from_toml(&text).map_err(|source| StoreError::Parse {
            path: path.to_path_buf(),
            source,
        }),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Config::default()),
        Err(e) => Err(io_err(path)(e)),
    }
}

/// Write atomically: temp file next to the target, then rename.
fn write_atomic(path: &Path, content: &str) -> Result<(), StoreError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(io_err(parent))?;
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, content).map_err(io_err(&tmp))?;
    std::fs::rename(&tmp, path).map_err(io_err(path))
}

pub fn save_config(path: &Path, config: &Config) -> Result<(), StoreError> {
    write_atomic(path, &config.to_toml()?)
}

/// Load the history; a missing or unreadable file gives an empty history.
pub fn load_history(path: &Path) -> History {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    History::from_lines(text.lines().map(str::to_owned))
}

pub fn save_history(path: &Path, history: &History) -> Result<(), StoreError> {
    let mut text = history.entries().join("\n");
    text.push('\n');
    write_atomic(path, &text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xdg_paths() {
        let p = Paths::in_dirs(Path::new("/c"), Path::new("/s"));
        assert_eq!(p.config_file, PathBuf::from("/c/multigit/config.toml"));
        assert_eq!(p.history_file, PathBuf::from("/s/multigit/history"));
    }

    #[test]
    fn missing_config_is_default_and_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::in_dirs(dir.path(), dir.path());
        assert_eq!(load_config(&paths.config_file).unwrap(), Config::default());
        let config = Config {
            repos: vec![PathBuf::from("/x")],
            ..Config::default()
        };
        save_config(&paths.config_file, &config).unwrap();
        assert_eq!(load_config(&paths.config_file).unwrap(), config);
    }

    #[test]
    fn broken_config_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "repos = [").unwrap();
        assert!(matches!(load_config(&path), Err(StoreError::Parse { .. })));
    }

    #[test]
    fn history_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("multigit/history");
        assert!(load_history(&path).entries().is_empty());
        let history = History::from_lines(["status".to_owned(), "pull".to_owned()]);
        save_history(&path, &history).unwrap();
        assert_eq!(load_history(&path), history);
    }
}

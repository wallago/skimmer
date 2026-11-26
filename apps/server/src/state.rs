//! Last-run timestamps, persisted between runs.

use std::collections::HashMap;
use std::fs::{create_dir_all, read_to_string};
use std::path::PathBuf;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use dirs::state_dir;
use serde::{Deserialize, Serialize};
use toml::value::Datetime;

use crate::prelude::*;

/// What we remember about one topic.
#[derive(Deserialize, Serialize)]
struct TopicState {
    /// When this topic was last briefed.
    last_run: Datetime,
}

/// The state file's contents.
#[derive(Deserialize, Serialize, Default)]
struct Data {
    /// Per-topic state, keyed by topic name.
    topic: HashMap<String, TopicState>,
}

/// The state file, loaded.
pub(crate) struct State {
    /// What was read from disk, plus anything set since.
    data: Data,
    /// Where it came from, and where [`save`](State::save) writes it back.
    path: PathBuf,
}

impl State {
    /// Reads the state file, creating its directory if needed.
    ///
    /// A missing or unreadable file is a first run, not an error.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if there is no state directory, or the directory
    /// cannot be created.
    pub(crate) fn load(path: Option<PathBuf>) -> Result<Self> {
        let path = path.unwrap_or(
            state_dir()
                .ok_or(Error::StatePath)?
                .join(env!("CARGO_PKG_NAME"))
                .join("state.toml"),
        );
        if let Some(dir) = path.parent() {
            create_dir_all(dir)?;
        }
        let contents = read_to_string(&path).unwrap_or_default();
        let data = toml::from_str(&contents).unwrap_or_else(|_| {
            tracing::trace!("State file has never been loaded");
            Data::default()
        });
        Ok(Self { data, path })
    }

    /// Getting topic last briefed, or [`None`] if it never was.
    pub(crate) fn get_last_run(&self, name: &String) -> Option<DateTime<Utc>> {
        let last_run = self.data.topic.get(name)?.last_run.to_string();
        DateTime::from_str(&last_run).ok()
    }

    /// Marks this topic as briefed up to now.    
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the timestamp does not parse as a TOML datetime.
    pub(crate) fn set_last_run(&mut self, name: &str) -> Result<()> {
        let last_run = Datetime::from_str(&Utc::now().to_rfc3339())?;
        self.data
            .topic
            .insert(name.to_owned(), TopicState { last_run });
        Ok(())
    }

    /// Writes the state back to disk.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if serializing or writing fails.
    pub(crate) fn save(&self) -> Result<()> {
        let contents = toml::to_string::<Data>(&self.data)?;
        Ok(std::fs::write(&self.path, contents)?)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::remove_dir_all, path::Path};

    use super::*;

    /// A state file that matches [`State`].
    const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/state.toml");

    /// A state file that is TOML, but not a valid [`State`].
    const INVALID_FIXTURE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/state_invalid.toml"
    );

    #[test]
    fn load_state() {
        let state = State::load(Some(Path::new(FIXTURE).to_path_buf())).unwrap();
        let last_run = state.get_last_run(&"rust".to_string()).unwrap();
        assert_eq!(last_run.to_rfc3339(), "2026-09-02T06:52:27.464906407+00:00");
        assert!(state.get_last_run(&"unknown".to_string()).is_none());
        assert!(state.get_last_run(&"fail".to_string()).is_none());
    }

    #[test]
    fn load_state_from_invalid_file() {
        let state = State::load(Some(PathBuf::from(INVALID_FIXTURE))).unwrap();
        assert!(state.data.topic.is_empty());
    }

    #[test]
    fn save_then_reload_state() {
        let state_file = temp_file("round-trip");
        let mut state = State::load(Some(state_file.clone())).unwrap();
        state.set_last_run("rust").unwrap();
        state.save().unwrap();

        let reloaded = State::load(Some(state_file.clone())).unwrap();
        assert!(reloaded.get_last_run(&"rust".to_string()).is_some());
        remove_temp_file(&state_file);
    }

    /// A scratch state file of our own, so tests never touch the real one.
    fn temp_file(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("skimmer-test-{name}-{}", std::process::id()));
        let _ = remove_dir_all(&dir);
        dir.join("state.toml")
    }

    /// Drops the scratch dir `temp_file` handed out.
    fn remove_temp_file(path: &Path) {
        remove_dir_all(path.parent().unwrap()).unwrap();
    }
}

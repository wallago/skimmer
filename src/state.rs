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
    pub(crate) fn load() -> Result<Self> {
        let path = state_dir()
            .ok_or(Error::StatePath)?
            .join(env!("CARGO_PKG_NAME"))
            .join("state.toml");
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

use std::collections::HashMap;
use std::fs::{create_dir_all, read_to_string};
use std::path::PathBuf;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use dirs::state_dir;
use serde::{Deserialize, Serialize};
use toml::value::Datetime;

use crate::{config, prelude::*, topic};

#[derive(Deserialize, Serialize)]
struct TopicState {
    last_run: Datetime,
}

#[derive(Deserialize, Serialize, Default)]
struct Data {
    topic: HashMap<String, TopicState>,
}

pub(crate) struct State {
    data: Data,
    path: PathBuf,
}

impl State {
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

    pub(crate) fn get_last_run(&self, topic: &config::topic::Topic) -> Option<DateTime<Utc>> {
        let last_run = self.data.topic.get(topic.get_name())?.last_run.to_string();
        DateTime::from_str(&last_run).ok()
    }

    pub(crate) fn set_last_run(&mut self, topic: &topic::Topic) -> Result<()> {
        let last_run = Datetime::from_str(&Utc::now().to_rfc3339())?;
        self.data
            .topic
            .insert(topic.get_name().to_string(), TopicState { last_run });
        Ok(())
    }

    pub(crate) fn save(&self) -> Result<()> {
        let contents = toml::to_string::<Data>(&self.data)?;
        Ok(std::fs::write(&self.path, contents)?)
    }
}

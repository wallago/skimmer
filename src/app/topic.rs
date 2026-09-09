//! Topics, resolved against the feed list and the state file.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use miniflux_api::models::Feed;

use crate::{
    ext::prelude::*,
    prelude::{Topic as TopicExt, *},
    state::State,
};

/// A configured topic.
pub(crate) struct Topic {
    /// Start of the window: entries published after this are read.
    last_run: DateTime<Utc>,
    /// Feeds matching this topic, keyed by title.
    feeds: HashMap<String, Feed>,
    /// The question put to Claude.
    question: String,
    /// The topic's name, as in the config and the state file.
    name: String,
}

impl Topic {
    /// Resolves every configured topic against the server's feeds.
    ///
    /// The window starts at `since` if given, else where the last run left off,
    /// else one `interval` ago.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if a topic's `interval` does not parse.
    pub(crate) fn new(
        cfg_topics: &HashMap<String, TopicExt>,
        since: Option<DateTime<Utc>>,
        state: &State,
        rss: &Rss,
    ) -> Result<HashMap<String, Self>> {
        let mut topics = HashMap::new();
        for (name, topic) in cfg_topics {
            let last_run = match since.or(state.get_last_run(name)) {
                Some(last_run) => last_run,
                None => Utc::now() - topic.get_interval()?,
            };
            let topic_feeds: HashMap<String, Feed> = rss
                .get_feeds()
                .iter()
                .filter(|feed| topic.get_feeds().iter().any(|p| feed.title.contains(p)))
                .map(|feed| (feed.title.clone(), feed.clone()))
                .collect();
            topics.insert(
                name.clone(),
                Topic {
                    last_run,
                    feeds: topic_feeds,
                    question: topic.get_question().to_string(),
                    name: name.to_owned(),
                },
            );
        }
        Ok(topics)
    }

    /// Start of this run's window.
    pub(crate) fn get_last_run(&self) -> &DateTime<Utc> {
        &self.last_run
    }

    /// The feeds this topic reads, keyed by title.
    pub(crate) fn get_feeds(&self) -> &HashMap<String, Feed> {
        &self.feeds
    }

    /// The question put to Claude.
    pub(crate) fn get_question(&self) -> &str {
        &self.question
    }

    /// The topic's name.
    pub(crate) fn get_name(&self) -> &str {
        &self.name
    }
}

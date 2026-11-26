//! Topics, resolved against the feed list and the last runs in the store.

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use super::store::Store;
use crate::{
    ext::prelude::*,
    prelude::{Result, Topic as TopicExt},
};

/// A configured topic.
pub(crate) struct Topic {
    /// Start of the window: entries published after this are read.
    last_run: DateTime<Utc>,
    /// Feeds matching this topic, keyed by title.
    feeds: HashMap<String, Feed>,
    /// The question put to Claude.
    question: String,
    /// What the reader wants out of this topic. Empty when unset.
    context: String,
    /// The topic's name, as in the config and the store.
    name: String,
    /// Web search limits, when the topic also searches the web.
    web_search: Option<WebSearch>,
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
        store: &Store,
        rss: &Rss,
    ) -> Result<HashMap<String, Self>> {
        let mut topics = HashMap::new();
        for (name, topic) in cfg_topics {
            let last_run = match since.or(store.last_run(name)?) {
                Some(last_run) => last_run,
                None => Utc::now() - topic.get_interval()?,
            };
            let topic_feeds: HashMap<String, Feed> = rss
                .get_feeds()
                .iter()
                .filter(|feed| topic.get_feeds().iter().any(|p| feed.title.contains(p)))
                .map(|feed| (feed.title.clone(), feed.clone()))
                .collect();
            if topic_feeds.is_empty() && !topic.get_feeds().is_empty() {
                let mut known: Vec<&str> = rss
                    .get_feeds()
                    .iter()
                    .map(|feed| feed.title.as_str())
                    .collect();
                known.sort_unstable();
                tracing::warn!(
                    "Topic {name}: no Miniflux feed title contains any of {:?}. Feed titles are: {known:?}",
                    topic.get_feeds(),
                );
            }
            topics.insert(
                name.clone(),
                Topic {
                    last_run,
                    feeds: topic_feeds,
                    question: topic.get_question().to_string(),
                    context: topic.get_context().to_string(),
                    name: name.to_owned(),
                    web_search: topic.get_web_search().map(|(max_uses, sites)| WebSearch {
                        max_uses,
                        allowed_domains: sites.to_vec(),
                    }),
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

    /// What the reader wants out of this topic. Empty when unset.
    pub(crate) fn get_context(&self) -> &str {
        &self.context
    }

    /// Web search limits, or [`None`] when the topic reads only its feeds.
    pub(crate) fn get_web_search(&self) -> Option<&WebSearch> {
        self.web_search.as_ref()
    }
}

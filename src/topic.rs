use std::collections::HashMap;

use chrono::{DateTime, Utc};
use miniflux_api::models::Feed;

use crate::{config, prelude::*, rss::Rss, state::State};

pub(crate) struct Topic {
    last_run: DateTime<Utc>,
    feeds: HashMap<String, Feed>,
    question: String,
    name: String,
}

impl Topic {
    pub(crate) fn new(
        cfg_topics: Vec<config::topic::Topic>,
        since: Option<DateTime<Utc>>,
        state: &State,
        rss: &Rss,
    ) -> Result<HashMap<String, Self>> {
        let mut topics = HashMap::new();
        for topic in &cfg_topics {
            let last_run = since.or(state.get_last_run(topic)).unwrap_or({
                let interval = topic.get_interval()?;
                Utc::now() - interval
            });
            let topic_feeds: HashMap<String, Feed> = rss
                .get_feeds()
                .iter()
                .filter(|feed| topic.get_feeds().iter().any(|p| feed.title.contains(p)))
                .map(|feed| (feed.title.clone(), feed.clone()))
                .collect();
            topics.insert(
                topic.get_name().to_string(),
                Topic {
                    last_run,
                    feeds: topic_feeds,
                    question: topic.get_question().to_string(),
                    name: topic.get_name().to_string(),
                },
            );
        }
        Ok(topics)
    }

    pub(crate) fn get_last_run(&self) -> &DateTime<Utc> {
        &self.last_run
    }

    pub(crate) fn get_feeds(&self) -> &HashMap<String, Feed> {
        &self.feeds
    }

    pub(crate) fn get_question(&self) -> &str {
        &self.question
    }

    pub(crate) fn get_name(&self) -> &str {
        &self.name
    }
}

//! A topic's briefing, as saved in the store.

use chrono::{DateTime, Utc};
use serde::Serialize;

use super::topic::Topic;
use crate::ext::prelude::{Analysis, Entry};

/// One entry as it appears in a report's source list.
#[derive(Serialize)]
pub(super) struct Source {
    /// Miniflux entry id, matched against `Highlight::entry_ids`.
    pub(super) id: i64,
    /// Entry title.
    pub(super) title: String,
    /// Title of the feed it came from.
    pub(super) feed: String,
    /// Link to the entry.
    pub(super) url: String,
}

/// One topic's section: what was asked, what Claude said, what it read.
pub(crate) struct Briefing {
    /// The topic's name, as in `[topic.<name>]`.
    pub(super) topic: String,
    /// The question put to Claude.
    pub(super) question: String,
    /// Start of the window these entries came from.
    pub(super) since: DateTime<Utc>,
    /// Claude's digest and highlights.
    pub(super) analysis: Analysis,
    /// Every entry scanned, for the sources block and the highlight links.
    pub(super) sources: Vec<Source>,
    /// How many entries were scanned.
    pub(super) scanned: usize,
}

impl Briefing {
    /// One topic's briefing, keeping the entries so highlights can link back
    /// to them.
    pub(super) fn new(topic: &Topic, entries: &[Entry], analysis: Analysis) -> Self {
        Self {
            topic: topic.get_name().to_string(),
            question: topic.get_question().to_string(),
            since: *topic.get_last_run(),
            scanned: entries.len(),
            analysis,
            sources: entries
                .iter()
                .map(|e| Source {
                    id: e.id,
                    title: e.title.clone(),
                    feed: e.feed.title.clone(),
                    url: e.url.clone(),
                })
                .collect(),
        }
    }
}

//! Render output.

use std::fmt::Write as WriteFmt;

use chrono::{DateTime, Utc};

use crate::ext::prelude::Analysis;

/// One entry as it appears in a report's source list.
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
    /// Renders one topic as a markdown section.
    pub(super) fn render(&self) -> String {
        let mut out = format!(
            "## {}\n\n_Since {} · {} entries scanned_\n\n{}\n\n**Highlights**\n\n",
            self.question,
            self.since.to_rfc2822(),
            self.scanned,
            self.analysis.get_digest(),
        );

        for highlight in self.analysis.get_highlights() {
            let links = highlight
                .get_entry_ids()
                .iter()
                .filter_map(|id| self.sources.iter().find(|source| source.id == *id))
                .map(|source| format!("[src]({})", source.url))
                .collect::<Vec<String>>()
                .join(" ");
            let _ = writeln!(
                out,
                "- **{}** — {} {}",
                highlight.get_headline(),
                highlight.get_detail(),
                links
            );
        }

        let _ = write!(
            out,
            "\n<details><summary>Sources — {} entries</summary>\n\n",
            self.scanned
        );
        for source in &self.sources {
            let _ = writeln!(
                out,
                "- [{}]({}) — {}\n",
                source.title, source.url, source.feed
            );
        }
        out.push_str("</details>\n");
        out
    }
}

//! The markdown report: one section per topic, written to a file.

use std::fmt::Write as WriteFmt;
use std::fs::File;
use std::io::Write as WriteIO;

use chrono::{DateTime, Utc};
use miniflux_api::models::Entry;

use super::topic::Topic;
use crate::{
    ext::prelude::{Claude, *},
    prelude::*,
};

/// One entry as it appears in a report's source list.
struct Source {
    /// Miniflux entry id, matched against `Highlight::entry_ids`.
    id: i64,
    /// Entry title.
    title: String,
    /// Title of the feed it came from.
    feed: String,
    /// Link to the entry.
    url: String,
}

/// One topic's section: what was asked, what Claude said, what it read.
pub(crate) struct Briefing {
    /// The question put to Claude.
    question: String,
    /// Start of the window these entries came from.
    since: DateTime<Utc>,
    /// Claude's digest and highlights.
    analysis: Analysis,
    /// Every entry scanned, for the sources block and the highlight links.
    sources: Vec<Source>,
    /// How many entries were scanned.
    scanned: usize,
}

impl Briefing {
    /// Renders one topic as a markdown section.
    fn render(&self) -> String {
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

/// Every topic's briefing, ready to write out.
pub(crate) struct Report {
    /// One per topic, in the order they were added.
    briefings: Vec<Briefing>,
}

impl Report {
    /// An empty report.
    pub(crate) fn new() -> Self {
        Self {
            briefings: Vec::new(),
        }
    }

    /// Writes the whole report to `report_<timestamp>.trash.md` in the working
    /// directory.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the file cannot be created or written.
    pub(crate) fn generate(&self, claude: &Claude) -> Result<()> {
        let now = Utc::now();
        let mut out = format!(
            "# Report — {}\n\n_{}_\n\n",
            now.to_rfc2822(),
            claude.get_model(),
        );
        for briefing in &self.briefings {
            out.push_str(&briefing.render());
            out.push('\n');
        }

        let stamp = now.format("%Y-%m-%d_%H-%M-%S");
        File::create(format!("report_{stamp}.trash.md"))?.write_all(out.as_bytes())?;
        Ok(())
    }

    /// Adds one topic's briefing, keeping the entries so highlights can link
    /// back to them.
    pub(crate) fn add(&mut self, topic: &Topic, entries: &[Entry], analysis: Analysis) {
        self.briefings.push(Briefing {
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
        });
    }
}

use std::fs::File;
use std::io::prelude::*;

use chrono::{DateTime, Utc};
use miniflux_api::models::Entry;

use crate::claude::Claude;
use crate::claude::output::Analysis;
use crate::prelude::*;
use crate::topic::Topic;

struct Highlight {
    headline: String,
    detail: String,
    // links: Vec<Url>,
}

struct Source {
    id: i64,
    title: String,
    feed: String,
    url: String,
}

pub(crate) struct Briefing {
    question: String,
    since: DateTime<Utc>,
    analysis: Analysis,
    sources: Vec<Source>,
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
            out.push_str(&format!(
                "- **{}** — {} {}\n",
                highlight.get_headline(),
                highlight.get_detail(),
                links
            ));
        }

        out.push_str(&format!(
            "\n<details><summary>Sources — {} entries</summary>\n\n",
            self.scanned
        ));
        for source in &self.sources {
            out.push_str(&format!(
                "- [{}]({}) — {}\n",
                source.title, source.url, source.feed
            ));
        }
        out.push_str("</details>\n");
        out
    }
}

pub(crate) struct Report {
    briefings: Vec<Briefing>,
}

impl Report {
    pub(crate) fn new() -> Self {
        Self {
            briefings: Vec::new(),
        }
    }

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

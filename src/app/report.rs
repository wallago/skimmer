//! The markdown report: one section per topic, written to a file.

use std::io::Write as WriteIO;
use std::{fs::File, path::PathBuf};

use chrono::Utc;
use miniflux_api::models::Entry;

use super::topic::Topic;
use crate::{
    app::render::{Briefing, Source},
    ext::prelude::{Claude, *},
    prelude::*,
};

/// Every topic's briefing, ready to write out.
pub(crate) struct Report {
    /// One per topic, in the order they were added.
    briefings: Vec<Briefing>,
    /// Output path.
    output_dir: PathBuf,
}

impl Report {
    /// An empty report.
    pub(crate) fn new(output_dir: PathBuf) -> Result<Self> {
        if !output_dir.exists() {
            return Err(Error::OutputNotExist);
        }
        Ok(Self {
            output_dir,
            briefings: Vec::new(),
        })
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
        let path = self.output_dir.join(format!("report_{stamp}.md"));
        File::create(path)?.write_all(out.as_bytes())?;
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

//! The report: one section per topic, written out as markdown and as a
//! self-contained HTML page.

use std::fmt::Write as WriteFmt;
use std::io::Write as WriteIO;
use std::{fs::File, path::PathBuf};

use chrono::{DateTime, Utc};

use super::topic::Topic;
use crate::app::render::escape;
use crate::{
    app::render::{Briefing, Source},
    ext::prelude::{Claude, *},
    prelude::*,
};

/// The page around the briefings, with `{{placeholder}}` holes to fill.
const TEMPLATE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/report.html"));

/// The report's stylesheet, inlined into the page so it stays one file.
const STYLE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/report.css"));

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

    /// Writes the whole report to `report_<timestamp>.md` and
    /// `report_<timestamp>.html` in the output directory.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the file cannot be created or written.
    pub(crate) fn generate(&self, claude: &Claude) -> Result<()> {
        let now = Utc::now();
        let stamp = now.format("%Y-%m-%d_%H-%M-%S");
        let mut out = format!(
            "# Report — {}\n\n_{}_\n\n",
            now.to_rfc2822(),
            claude.get_model(),
        );
        for briefing in &self.briefings {
            out.push_str(&briefing.render());
            out.push('\n');
        }

        let path = self.output_dir.join(format!("report_{stamp}.md"));
        File::create(path)?.write_all(out.as_bytes())?;

        let path = self.output_dir.join(format!("report_{stamp}.html"));
        File::create(path)?.write_all(self.render_html(claude, now).as_bytes())?;
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

    /// The whole report as one self-contained HTML page: no scripts, no fonts,
    /// no network at all once it is written.
    fn render_html(&self, claude: &Claude, now: DateTime<Utc>) -> String {
        let mut body = String::from("<nav>\n<ol>\n");

        for (index, briefing) in self.briefings.iter().enumerate() {
            let _ = writeln!(
                body,
                "<li><a href=\"#topic-{index}\">{}</a></li>",
                escape(&briefing.question)
            );
        }
        body.push_str("</ol>\n</nav>\n");

        for (index, briefing) in self.briefings.iter().enumerate() {
            body.push_str(&briefing.render_html(index));
        }

        TEMPLATE
            .replace("{{style}}", STYLE)
            .replace("{{date}}", &escape(&now.format("%-d %B %Y").to_string()))
            .replace("{{model}}", &escape(claude.get_model()))
            .replace("{{body}}", &body)
    }
}

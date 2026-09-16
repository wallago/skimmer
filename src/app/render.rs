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

    /// Renders one topic as an HTML section, anchored on `index` for the index
    /// links at the top of the document.
    pub(super) fn render_html(&self, index: usize) -> String {
        let mut out = format!(
            "<section id=\"topic-{index}\">\n<h2>{}</h2>\n<p class=\"meta\">Since {} · {} entries scanned</
p>\n",
            escape(&self.question),
            escape(&self.since.to_rfc2822()),
            self.scanned,
        );

        for paragraph in self.analysis.get_digest().split("\n\n") {
            let _ = writeln!(out, "<p>{}</p>", escape(paragraph.trim()));
        }

        out.push_str("<h3>Highlights</h3>\n<ul class=\"highlights\">\n");
        for highlight in self.analysis.get_highlights() {
            let links = highlight
                .get_entry_ids()
                .iter()
                .filter_map(|id| self.sources.iter().find(|source| source.id == *id))
                .map(|source| format!("<a href=\"{}\">src</a>", escape(&source.url)))
                .collect::<Vec<String>>()
                .join(" ");
            let _ = writeln!(
                out,
                "<li><strong>{}</strong> {} <span class=\"links\">{}</span></li>",
                escape(highlight.get_headline()),
                escape(highlight.get_detail()),
                links
            );
        }
        out.push_str("</ul>\n");

        let _ = write!(
            out,
            "<details>\n<summary>Sources — {} entries</summary>\n<ul class=\"sources\">\n",
            self.scanned
        );
        for source in &self.sources {
            let _ = writeln!(
                out,
                "<li><a href=\"{}\">{}</a> <span class=\"feed\">{}</span></li>",
                escape(&source.url),
                escape(&source.title),
                escape(&source.feed)
            );
        }
        out.push_str("</ul>\n</details>\n</section>\n");
        out
    }
}

/// Escapes the characters that would otherwise break out of HTML text or a
/// quoted attribute.
pub(crate) fn escape(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for char in raw.chars() {
        match char {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(char),
        }
    }
    out
}

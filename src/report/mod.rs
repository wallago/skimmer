use std::fs::File;
use std::io::prelude::*;

use chrono::{DateTime, Duration, Utc};

use crate::claude::{Claude, Resp};
use crate::prelude::*;

struct Highlight {
    headline: String,
    detail: String,
    // links: Vec<Url>,
}

struct Source {
    title: String,
    feed: String,
    // url: Url,
}

pub(crate) struct Briefing {
    question: String,
    digest: String,
    highlights: Vec<Highlight>,
    sources: Vec<Source>,
    scanned: usize,
}

pub(crate) struct Report {
    interval: Duration,
    last_poll: DateTime<Utc>,
    briefings: Vec<Briefing>,
}

impl Report {
    pub(crate) fn new() -> Self {
        Self {
            interval: Duration::hours(24),
            briefings: Vec::new(),
            last_poll: Utc::now(),
        }
    }

    fn template(&self, claude: Claude) -> String {
        let now = Utc::now();
        format!(
            "
        # Report — {}

        _Past {} · {}_

        ## What's interesting in the Rust world this past day?

            Two or three paragraphs of synthesis: the themes, what's new, what's
            noise. Written by Claude from the entries below.

            **Highlights**
            - **Async Drop hits nightly** — closes a long-standing soundness gap.
            [src](url)
            - **This Week in Rust #560** — notable RFCs on… [src](url)

            <details><summary>Sources — 23 entries</summary>

            - [Announcing Rust 1.x](url) — Rust Blog
            - [title](url) — Lobsters
            - …
            </details>

            ## What's new in the bicycle world?
            …
            ",
            now.to_rfc2822(),
            self.last_poll - now,
            claude.get_model(),
        )
    }

    pub(crate) fn generate(&self, claude: Claude) -> Result<()> {
        let header = self.template(claude);
        let mut file = File::create(format!("report_{}.trash.md", Utc::now().to_rfc3339()))?;
        file.write_all(header.as_bytes())?;
        Ok(())
    }

    pub(crate) fn add(&mut self, resp: Resp) {
        for value in resp.get_content() {
            // self.content.push(value.to_string());
        }
    }
}

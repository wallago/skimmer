//! Miniflux, the feed source.

use miniflux_api::{
    MinifluxApi,
    models::{Entry, Feed},
};
use reqwest::Client;
use url::Url;

use crate::prelude::*;

/// A connection to Miniflux, with the feed list fetched once at startup.
pub(crate) struct Rss {
    /// Miniflux API bindings.
    miniflux: MinifluxApi,
    /// HTTP client, reused across requests.
    client: Client,
    /// Every feed on the server, fetched in [`new`](Rss::new).
    feeds: Vec<Feed>,
}

impl Rss {
    /// Connects to Miniflux and fetches the feed list.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the server is unreachable or rejects the
    /// credentials.
    pub(crate) async fn new(url: Url, username: String, password: String) -> Result<Self> {
        let miniflux = MinifluxApi::new(&url, username, password);
        let client = Client::new();
        let feeds = miniflux.get_feeds(&client).await?;
        Ok(Self {
            miniflux,
            client,
            feeds,
        })
    }

    /// Every feed on the server. Topics match against these by title.
    pub(crate) fn get_feeds(&self) -> &[Feed] {
        &self.feeds
    }

    /// Entries published after the `after` unix timestamp, newest 200 at most.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the request fails.
    pub(crate) async fn get_entries(&self, feed: i64, after: i64) -> Result<Vec<Entry>> {
        Ok(self
            .miniflux
            .get_feed_entries(
                feed,
                None,
                None,
                Some(200),
                None,
                None,
                None,
                Some(after),
                None,
                None,
                None,
                &self.client,
            )
            .await?)
    }

    /// Lays entries out as XML-ish blocks for the prompt. The `id` attribute is
    /// what Claude cites highlights with.
    pub(crate) fn render_entries(entries: &[Entry]) -> String {
        entries.iter().map(|e| format!(
        "<entry id=\"{}\" feed=\"{}\" published=\"{}\">\n<title>{}</title>\n<content>{}</content>\n</entry>",
         e.id, e.feed.title, e.published_at, e.title, escape(&e.content),
        )).collect::<Vec<_>>().join("\n")
    }
}

/// Escapes the XML metacharacters, so feed content cannot close the block it
/// sits in.
fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

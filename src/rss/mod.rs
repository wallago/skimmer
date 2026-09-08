use miniflux_api::{
    MinifluxApi,
    models::{Entry, Feed},
};
use reqwest::Client;
use url::Url;

use crate::prelude::*;

pub(crate) struct Rss {
    miniflux: MinifluxApi,
    client: Client,
    feeds: Vec<Feed>,
}

impl Rss {
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

    pub(crate) fn get_feeds(&self) -> &[Feed] {
        &self.feeds
    }

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

    pub(crate) fn render_entries(entries: &[Entry]) -> String {
        entries.iter().map(|e| format!(
        "<entry id=\"{}\" feed=\"{}\" published=\"{}\">\n<title>{}</title>\n<content>{}</content>\n</entry>",
         e.id, e.feed.title, e.published_at, e.title, escape(&e.content),
        )).collect::<Vec<_>>().join("\n")
    }
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

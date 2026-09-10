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

#[cfg(test)]
mod tests {
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    /// Entries as Miniflux returns them.
    const ENTRIES: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/entries.json"
    ));

    /// Feeds as Miniflux returns them.
    const FEEDS: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/feeds.json"
    ));

    fn entries() -> Vec<Entry> {
        serde_json::from_str(ENTRIES).unwrap()
    }

    /// A mock Miniflux answering `/v1/feeds`; mount anything else on top.
    async fn mock_server() -> MockServer {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/feeds"))
            .respond_with(ResponseTemplate::new(200).set_body_string(FEEDS))
            .mount(&server)
            .await;
        server
    }

    /// Connects to `server`, which must answer `/v1/feeds`.
    async fn connect(server: &MockServer) -> Rss {
        Rss::new(
            Url::parse(&server.uri()).unwrap(),
            "admin".into(),
            "password".into(),
        )
        .await
        .unwrap()
    }

    #[test]
    fn escapes_xml_metacharacters() {
        assert_eq!(escape("a & b"), "a &amp; b");
        assert_eq!(escape("</entry>"), "&lt;/entry&gt;");
        assert_eq!(escape("plain"), "plain");
        assert_eq!(escape("&lt;"), "&amp;lt;");
    }

    #[test]
    fn renders_entries_as_blocks() {
        let rendered = Rss::render_entries(&entries());
        assert!(rendered.starts_with(r#"<entry id="42" feed="This Week in Rust""#));
        assert!(rendered.contains("<content>Tom &amp; Jerry &lt;script&gt;"));
        assert!(!rendered.contains("<script>"));
    }

    #[test]
    fn renders_nothing_for_no_entries() {
        assert_eq!(Rss::render_entries(&[]), "");
    }

    #[tokio::test]
    async fn fetches_feeds_on_connect() {
        let server = mock_server().await;
        let rss = connect(&server).await;
        assert_eq!(rss.get_feeds().len(), 3);
        assert_eq!(rss.get_feeds()[0].title, "This Week in Rust");
    }

    #[tokio::test]
    async fn get_entries_asks_for_the_window() {
        let server = mock_server().await;
        let body = format!("{{\"total\": 1, \"entries\": {ENTRIES}}}");
        Mock::given(method("GET"))
            .and(path("/v1/feeds/7/entries"))
            .and(query_param("limit", "200"))
            .and(query_param("after", "1756795947"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .expect(1)
            .mount(&server)
            .await;

        let entries = connect(&server)
            .await
            .get_entries(7, 1_756_795_947)
            .await
            .unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, 42);
    }

    #[tokio::test]
    async fn rejects_bad_credentials() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;

        assert!(
            Rss::new(Url::parse(&server.uri()).unwrap(), "a".into(), "b".into())
                .await
                .is_err()
        );
    }
}

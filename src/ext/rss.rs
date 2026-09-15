//! Miniflux, the feed source.

use reqwest::{Client, StatusCode};
use serde::Deserialize;
use url::Url;

use crate::prelude::*;

/// A feed on the Miniflux server. Only the fields we read are modelled; the
/// server sends many more and serde drops them.
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Feed {
    /// Miniflux' feed id, used to ask for the feed's entries.
    pub(crate) id: i64,
    /// Display title, what topics match against.
    pub(crate) title: String,
}

/// An entry, as nested inside [`Entry`]. Only the title is used.
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct EntryFeed {
    /// Title of the feed the entry came from.
    pub(crate) title: String,
}

/// An entry on the Miniflux server.
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Entry {
    /// Miniflux' entry id, what Claude cites highlights with.
    pub(crate) id: i64,
    /// Headline.
    pub(crate) title: String,
    /// Body, as HTML.
    pub(crate) content: String,
    /// Link back to the article, cited in the report.
    pub(crate) url: String,
    /// RFC 3339 timestamp, passed through to the prompt as-is.
    pub(crate) published_at: String,
    /// The feed it came from.
    pub(crate) feed: EntryFeed,
}

/// The envelope `/v1/feeds/{id}/entries` wraps its entries in.
#[derive(Deserialize)]
struct EntryBatch {
    /// The entries themselves; `total` is ignored.
    entries: Vec<Entry>,
}

/// The shape Miniflux reports errors in.
#[derive(Deserialize)]
struct MinifluxError {
    /// Human-readable reason.
    error_message: String,
}

/// A connection to Miniflux, with the feed list fetched once at startup.
pub(crate) struct Rss {
    /// Base url of the Miniflux instance.
    base: Url,
    /// HTTP client, reused across requests.
    client: Client,
    /// Username, sent as HTTP basic auth on every request.
    username: String,
    /// Password, sent as HTTP basic auth on every request.
    password: String,
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
        let mut rss = Self {
            base: url,
            client: Client::new(),
            username,
            password,
            feeds: Vec::new(),
        };
        rss.feeds = rss.get("v1/feeds", &[]).await?;
        Ok(rss)
    }

    /// `GET`s `path` under the base url with `query` appended, authenticated
    /// and deserialized.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the request fails or the server answers with
    /// anything but `200 OK`.
    async fn get<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<T> {
        let response = self
            .client
            .get(self.base.join(path)?)
            .basic_auth(&self.username, Some(&self.password))
            .query(query)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;
        if status != StatusCode::OK {
            // Miniflux reports the reason in the body, but not for every
            // status; fall back to the code itself.
            let message = serde_json::from_str::<MinifluxError>(&body)
                .map_or_else(|_| status.to_string(), |e| e.error_message);
            return Err(Error::MinifluxApi(message));
        }
        Ok(serde_json::from_str(&body)?)
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
        let batch: EntryBatch = self
            .get(
                &format!("v1/feeds/{feed}/entries"),
                &[("limit", "200".to_owned()), ("after", after.to_string())],
            )
            .await?;
        Ok(batch.entries)
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

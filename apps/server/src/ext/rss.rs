//! Miniflux, the feed source.

use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
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

/// The body of `PUT /v1/entries`.
#[derive(Serialize)]
struct EntryStatusUpdate<'a> {
    /// Entries to update.
    entry_ids: &'a [i64],
    /// New status; always `"read"` here.
    status: &'a str,
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
    pub(crate) async fn new(url: Url, username: &str, password: &str) -> Result<Self> {
        let mut rss = Self {
            base: url,
            client: Client::new(),
            username: username.to_owned(),
            password: password.to_owned(),
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

        let body = Self::body(response, StatusCode::OK).await?;
        Ok(serde_json::from_str(&body)?)
    }

    /// Checks `response` against `expected`, handing back its body on success
    /// and Miniflux' own error message otherwise.
    async fn body(response: reqwest::Response, expected: StatusCode) -> Result<String> {
        let status = response.status();
        let body = response.text().await?;
        if status != expected {
            // Miniflux reports the reason in the body, but not for every
            // status; fall back to the code itself.
            let message = serde_json::from_str::<MinifluxError>(&body)
                .map_or_else(|_| status.to_string(), |e| e.error_message);
            return Err(Error::MinifluxApi(message));
        }
        Ok(body)
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

    /// Marks `entry_ids` as read on the server.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the request fails or Miniflux rejects it.
    pub(crate) async fn mark_as_read(&self, entry_ids: &[i64]) -> Result<()> {
        if entry_ids.is_empty() {
            return Ok(());
        }
        let response = self
            .client
            .put(self.base.join("v1/entries")?)
            .basic_auth(&self.username, Some(&self.password))
            .json(&EntryStatusUpdate {
                entry_ids,
                status: "read",
            })
            .send()
            .await?;
        Self::body(response, StatusCode::NO_CONTENT).await?;
        Ok(())
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
        Rss::new(Url::parse(&server.uri()).unwrap(), "admin", "password")
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
            Rss::new(Url::parse(&server.uri()).unwrap(), "a", "b")
                .await
                .is_err()
        );
    }
}

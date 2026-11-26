//! Per-topic settings.

use std::time::Duration;

use duration_str::parse;
use serde::Deserialize;

use crate::prelude::*;

/// One `[topic.x]` from the config: a question, and which feeds answer it.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Topic {
    /// What Claude is asked about this topic's entries.
    question: String,
    /// Feed title substrings; a feed matches if its title contains one.
    feeds: Vec<String>,
    /// How far back to look on a first run, as a duration like `"7d"`.
    interval: String,
    /// What the reader wants out of this topic specifically. Optional.
    #[serde(default)]
    context: String,
    /// Also search the web for this topic, on top of its feeds. Off by default.
    #[serde(default)]
    web_search: bool,
    /// Most web searches per run when `web_search` is on.
    #[serde(default = "default_max_searches")]
    max_searches: u32,
    /// Sites the web search may use, like `"blog.rust-lang.org"`. Empty means
    /// any site.
    #[serde(default)]
    allowed_sites: Vec<String>,
}

/// Three searches is about $0.03 a run, enough for a daily topic.
const fn default_max_searches() -> u32 {
    3
}

impl Topic {
    /// The question put to Claude.
    #[must_use]
    pub fn get_question(&self) -> &str {
        &self.question
    }
    /// The feed title substrings to match.
    #[must_use]
    pub fn get_feeds(&self) -> &[String] {
        &self.feeds
    }

    /// The configured interval, parsed.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if `interval` is not a duration `duration_str`
    /// understands.
    pub fn get_interval(&self) -> Result<Duration> {
        parse(&self.interval).map_err(Error::IntervalParsing)
    }

    /// What the reader wants out of this topic. Empty when unset.
    #[must_use]
    pub fn get_context(&self) -> &str {
        &self.context
    }

    /// The web search limits, or [`None`] when `web_search` is off.
    #[must_use]
    pub fn get_web_search(&self) -> Option<(u32, &[String])> {
        self.web_search
            .then_some((self.max_searches, self.allowed_sites.as_slice()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_deserializes_toml_list() {
        let modules: Topic = toml::from_str(
            "question = \"test ?\"\ncontext = \"test\"\nfeeds = [\"test\"]\ninterval = \"23h\"",
        )
        .unwrap();
        assert_eq!(modules.question, "test ?");
        assert_eq!(modules.context, "test");
        assert_eq!(modules.interval, "23h");
        assert_eq!(modules.feeds, ["test"].to_vec());
        assert!(
            toml::from_str::<Topic>(
                "question = [\"test ?\"]\ncontext = [\"test\"]\nfeeds = \"test\"\ninterval = [\"23h\"]",
            )
            .is_err()
        );
    }

    #[test]
    fn web_search_is_off_unless_asked_and_defaults_to_three_searches() {
        let base = "question = \"q\"\nfeeds = []\ninterval = \"1d\"\n";
        let off: Topic = toml::from_str(base).unwrap();
        assert_eq!(off.get_web_search(), None);
        let on: Topic = toml::from_str(&format!("{base}web_search = true")).unwrap();
        assert_eq!(on.get_web_search(), Some((3, [].as_slice())));
        let limited: Topic = toml::from_str(&format!(
            "{base}web_search = true\nmax_searches = 1\nallowed_sites = [\"blog.rust-lang.org\"]"
        ))
        .unwrap();
        assert_eq!(
            limited.get_web_search(),
            Some((1, ["blog.rust-lang.org".to_owned()].as_slice()))
        );
    }
}

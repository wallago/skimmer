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
}

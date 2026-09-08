use std::time::Duration;

use duration_str::parse;
use serde::Deserialize;

use crate::prelude::*;

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct Topic {
    name: String,
    question: String,
    feeds: Vec<String>,
    interval: String,
}

impl Topic {
    pub(crate) fn get_name(&self) -> &str {
        &self.name
    }

    pub(crate) fn get_question(&self) -> &str {
        &self.question
    }

    pub(crate) fn get_feeds(&self) -> &[String] {
        &self.feeds
    }

    pub(crate) fn get_interval(&self) -> Result<Duration> {
        Ok(parse(&self.interval).map_err(|err| Error::IntervalParsing(err))?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_deserializes_toml_list() {
        let modules: Topic = toml::from_str(
            "
                name = \"test\", 
                question = \"test ?\", 
                feeds = [\"test\"]",
        )
        .unwrap();
        assert_eq!(modules.name, "test");
        assert_eq!(modules.question, "test ?");
        assert_eq!(modules.feeds, ["test"].to_vec());
        assert!(
            toml::from_str::<Topic>(
                "
                name = [\"test\"], 
                question = [\"test ?\"], 
                feeds = \"test\"",
            )
            .is_err()
        );
    }
}

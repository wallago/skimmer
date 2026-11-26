//! Anthropic API settings.

use std::path::PathBuf;

use serde::Deserialize;

use crate::prelude::*;

/// The `[claude]` table: who to ask, and with what credentials.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Claude {
    /// Anthropic API key. Optional when `api_key_file` is set.
    #[serde(default)]
    api_key: String,
    /// File holding the API key.
    api_key_file: Option<PathBuf>,
    /// Model id.
    model: String,
}

impl Claude {
    /// The API key.
    #[must_use]
    pub fn get_key(&self) -> &str {
        &self.api_key
    }

    /// The model id.
    #[must_use]
    pub fn get_model(&self) -> &str {
        &self.model
    }

    /// Replaces `api_key` with the contents of `api_key_file`, if set.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read, or if neither `api_key` nor
    /// `api_key_file` is set.
    pub(super) fn resolve(&mut self) -> Result<()> {
        if let Some(path) = self.api_key_file.take() {
            self.api_key = read_secret(&path)?;
        }
        if self.api_key.is_empty() {
            return Err(Error::Config(
                "claude: set either `api_key` or `api_key_file`".to_owned(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_deserializes_toml_list() {
        let modules: Claude = toml::from_str(
            "
            api_key = \"test\"
            model = \"test\"
            ",
        )
        .unwrap();
        assert_eq!(modules.api_key, "test");
        assert_eq!(modules.model, "test");
        assert!(
            toml::from_str::<Claude>(
                "
                api_key = [\"test\"]
                model = [\"test\"]
                "
            )
            .is_err()
        );
    }

    #[test]
    fn should_accept_api_key_file_instead_of_api_key() {
        let mut claude: Claude = toml::from_str(&format!(
            "
            api_key_file = \"{}/tests/fixtures/secret\"
            model = \"test\"
            ",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        claude.resolve().unwrap();
        assert_eq!(claude.get_key(), "sk-file-key");
    }

    #[test]
    fn should_reject_missing_api_key_and_file() {
        let mut claude: Claude = toml::from_str("model = \"test\"").unwrap();
        assert!(claude.resolve().is_err());
    }
}

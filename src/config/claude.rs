//! Anthropic API settings.

use std::path::PathBuf;

use serde::Deserialize;

use crate::prelude::*;

/// The `[claude]` table: who to ask, and with what credentials.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Claude {
    /// Anthropic API key.
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
    /// Returns an error if the file cannot be read.
    pub(super) fn resolve(&mut self) -> Result<()> {
        if let Some(path) = self.api_key_file.take() {
            self.api_key = read_secret(&path)?;
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
}

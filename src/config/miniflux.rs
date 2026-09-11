//! Miniflux connection settings.

use std::path::PathBuf;

use serde::Deserialize;

use crate::prelude::*;

/// The `[miniflux]` table: where the feeds come from.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Miniflux {
    /// Base URL of the Miniflux server.
    url: String,
    /// Account to read feeds as.
    username: String,
    /// That account's password.
    password: String,
    /// File holding the password, e.g. a sops-nix secret path.
    password_file: Option<PathBuf>,
}

impl Miniflux {
    /// Consumes the config and hands back `(url, username, password)`.
    #[must_use]
    pub fn get_all(self) -> (String, String, String) {
        (self.url, self.username, self.password)
    }

    /// Replaces `password` with the contents of `password_file`, if set.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read.
    pub(super) fn resolve(&mut self) -> Result<()> {
        if let Some(path) = self.password_file.take() {
            self.password = read_secret(&path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_deserializes_toml_list() {
        let modules: Miniflux =
            toml::from_str("url = \"test\"\nusername = \"test_u\"\npassword = \"test_p\"").unwrap();
        assert_eq!(modules.url, "test");
        assert_eq!(modules.username, "test_u");
        assert_eq!(modules.password, "test_p");
        assert!(
            toml::from_str::<Miniflux>(
                "url = [\"test\"]\nusername = [\"test_u\"]\npassword = [\"test_p\"]"
            )
            .is_err()
        );
    }
}

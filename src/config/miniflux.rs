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
    /// That account's password. Optional when `password_file` is set.
    #[serde(default)]
    password: String,
    /// File holding the password, e.g. a sops-nix secret path.
    password_file: Option<PathBuf>,
}

impl Miniflux {
    /// Consumes the config and hands back `(url, username, password)`.
    #[must_use]
    pub fn get_all(&self) -> (&str, &str, &str) {
        (&self.url, &self.username, &self.password)
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
        if self.password.is_empty() {
            return Err(Error::Config(
                "miniflux: set either `password` or `password_file`".to_owned(),
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

    #[test]
    fn should_accept_password_file_instead_of_password() {
        let mut miniflux: Miniflux = toml::from_str(&format!(
            "
            password_file = \"{}/tests/fixtures/secret\"
            username = \"admin\"
            url = \"test\"
            ",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        miniflux.resolve().unwrap();
        let (_, _, password) = miniflux.get_all();
        assert_eq!(password, "sk-file-key");
    }

    #[test]
    fn should_reject_missing_password_and_file() {
        let mut miniflux: Miniflux =
            toml::from_str("username = \"admin\"\nurl = \"test\"").unwrap();
        assert!(miniflux.resolve().is_err());
    }
}

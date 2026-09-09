//! Miniflux connection settings.

use serde::Deserialize;

/// The `[miniflux]` table: where the feeds come from.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Miniflux {
    /// Base URL of the Miniflux server.
    url: String,
    /// Account to read feeds as.
    username: String,
    /// That account's password.
    password: String,
}

impl Miniflux {
    /// Consumes the config and hands back `(url, username, password)`.
    #[must_use]
    pub fn get_all(self) -> (String, String, String) {
        (self.url, self.username, self.password)
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

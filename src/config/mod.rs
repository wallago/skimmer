//! Config file setup.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use claude::Claude;
use miniflux::Miniflux;
use serde::Deserialize;
use topic::Topic;

use crate::prelude::*;

/// Anthropic API settings.
mod claude;

/// Miniflux connection settings.
mod miniflux;

/// Per-topic settings.
mod topic;

/// The config types, for modules that hold one.
pub mod prelude {
    pub use super::claude::Claude;
    pub use super::miniflux::Miniflux;
    pub use super::topic::Topic;
}

/// Configuration loaded from `config.toml`.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// Where the feeds come from.
    pub miniflux: Miniflux,
    /// Which model summarizes them, and with what key.
    pub claude: Claude,
    /// The topics to brief on, one `[topic.x]` each.
    pub topic: HashMap<String, Topic>,
    /// Output dir.
    pub output: PathBuf,
}

impl Config {
    /// Loads the config: explicit `--config` path, else
    /// `$XDG_CONFIG_HOME/env!("CARGO_PKG_NAME")/config.toml` if present, else defaults.
    ///  
    /// # Errors
    ///
    /// Returns an error if a config file is found but cannot be read or parsed.
    pub fn load(cli_path: Option<&Path>) -> Result<Self> {
        if let Some(path) = cli_path {
            return Self::from_file(path);
        }
        match dirs::config_dir()
            .map(|dir| dir.join(format!("{}/config.toml", env!("CARGO_PKG_NAME"))))
        {
            Some(path) if path.exists() => Self::from_file(&path),
            _ => Ok(Self::default()),
        }
    }

    /// Reads and parses a config file; any failure is a hard error.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read, or if its contents are not
    /// valid TOML for this config.
    pub fn from_file(path: &Path) -> Result<Self> {
        // Get raw content
        let raw = std::fs::read_to_string(path)
            .map_err(|error| Error::Config(format!("{}: {error}", path.display())))?;
        let mut config: Self = toml::from_str(&raw)
            .map_err(|error| Error::Config(format!("{}: {error}", path.display())))?;
        config.claude.resolve()?;
        config.miniflux.resolve()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use std::{path::Path, time::Duration};

    use pretty_assertions::assert_eq;

    use super::*;

    /// A config file that matches [`Config`].
    const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/config.toml");

    /// A config file that is TOML, but not a valid [`Config`].
    const INVALID_FIXTURE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/config_invalid.toml"
    );

    #[test]
    fn load_config_from_file() {
        let config = Config::from_file(Path::new(FIXTURE)).unwrap();
        check_fixture_config(config);
    }

    #[test]
    fn load_config_with_args() {
        let config = Config::load(Some(Path::new(FIXTURE))).unwrap();
        check_fixture_config(config);
    }

    #[test]
    fn load_config_without_args() {
        drop(Config::load(None));
    }

    #[test]
    fn load_config_from_missing_file() {
        let path = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/nope.toml"
        ));
        assert!(Config::from_file(path).is_err());
        assert!(Config::load(Some(path)).is_err());
    }

    #[test]
    fn load_config_from_invalid_file() {
        assert!(Config::from_file(Path::new(INVALID_FIXTURE)).is_err());
    }

    #[test]
    fn default_config_is_empty() {
        let config = Config::default();
        assert!(config.topic.is_empty());
        assert_eq!(config.claude.get_model(), "");
        assert_eq!(
            config.miniflux.get_all(),
            (String::new(), String::new(), String::new())
        );
    }

    fn check_fixture_config(config: Config) {
        // Topic
        let topic = config.topic.get("rust").unwrap();
        assert_eq!(
            topic.get_question(),
            "What's going on interesting in the Rust world the past day?"
        );
        assert_eq!(topic.get_feeds(), ["Reddit Rust", "This Week in Rust"]);
        assert_eq!(
            topic.get_interval().unwrap(),
            Duration::from_secs(7 * 24 * 60 * 60)
        );
        assert_eq!(config.topic.len(), 1);

        // Claude
        assert_eq!(config.claude.get_model(), "claude-haiku-4-5");
        assert_eq!(config.claude.get_key(), "sk-key");

        // Miniflux
        let (url, username, password) = config.miniflux.get_all();
        assert_eq!(url, "https://localhost");
        assert_eq!(username, "admin");
        assert_eq!(password, "password");
    }
}

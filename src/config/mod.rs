//! Config file setup.

use std::{collections::HashMap, path::Path};

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
        // Deserialize content in TOML format
        toml::from_str(&raw).map_err(|error| Error::Config(format!("{}: {error}", path.display())))
    }
}

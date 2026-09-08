//! Error handler implementation.

use thiserror::Error as ThisError;
use toml::value::DatetimeParseError;

/// Errors that can occur while running [`run`].
#[derive(Debug, ThisError)]
pub enum Error {
    /// Error that may occur during I/O operations.
    #[error("IO error: `{0}`")]
    Io(#[from] std::io::Error),
    /// Error that may occur while loading the application config file.
    #[error("Config file error: `{0}`")]
    Config(String),
    /// Error that occur while parsing url from string.
    #[error("Url parsing error: `{0}`")]
    UrlParsing(#[from] url::ParseError),
    /// Error that occur while processing web request.
    #[error("Reqwest error: `{0}`")]
    Reqwest(#[from] reqwest::Error),
    /// Error that occur while processing serialization.
    #[error("Serialize error: `{0}`")]
    Serialize(#[from] serde_json::Error),
    /// Error returned by the Anthropic API.
    #[error("Anthropic API error: `{0}`")]
    AnthropicApi(String),
    /// Error returned by the miniflux API.
    #[error("Miniflux API error: `{0}`")]
    MinifluxApi(#[from] miniflux_api::ApiError),
    /// Error may occur by founding state path.
    #[error("State path not found")]
    StatePath,
    /// Error may occur while de parsing toml.
    #[error("Toml de parsing error: `{0}`")]
    TomlDeParsing(#[from] toml::de::Error),
    /// Error may occur while ser parsing toml.
    #[error("Toml ser parsing error: `{0}`")]
    TomlSerParsing(#[from] toml::ser::Error),
    /// Error may occur while parsing interval.
    #[error("Interval parsing error: `{0}`")]
    IntervalParsing(String),
    /// Error may occur while date parsing toml.
    #[error("Toml date parsing error: `{0}`")]
    TomlDateParsing(#[from] DatetimeParseError),
}

/// Type alias for the standard [`Result`] type.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use std::io::Error as IoError;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_error() {
        let message = "your computer is on fire!";
        let error = Error::from(IoError::other(message));
        assert_eq!(format!("IO error: `{message}`"), error.to_string());
        assert_eq!(
            format!("\"IO error: `{message}`\""),
            format!("{:?}", error.to_string())
        );
    }
}

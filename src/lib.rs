//! AI agent that reads your RSS feeds and keeps only what's worth your time.

/// Error handler implementation.
pub mod error;

/// Main application.
pub mod app;

/// Command-line arguments parser.
pub mod args;

/// Claude API.
pub mod claude;

/// Config file.
pub mod config;

/// Helper functions.
pub mod help;

/// Common types that can be glob-imported for convenience.
pub mod prelude;

use std::str::FromStr;

use miniflux_api::MinifluxApi;
use prelude::*;
use tracing::info;

use crate::config::Config;

/// Runs `skimmer`.
///
/// # Errors
///
/// Returns an [`Error`] if the run fa
pub async fn run(args: &Args) -> Result<()> {
    better_panic::install();
    let config = Config::load(args.config.as_deref())?;
    let (url, username, password) = config.miniflux.get_all();
    // TODO
    // make a struct for miniflux and client
    let miniflux = MinifluxApi::new(&url::Url::from_str(&url)?, username, password);
    let client = reqwest::Client::new();
    let key = config.claude.get_key();
    claude::say_hello(key).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn run_succeeds() {
        let args = Args {
            config: Some(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/config.toml").into()),
            ..Args::default()
        };
        assert_eq!(run(&args).await.is_ok(), true);
    }
}

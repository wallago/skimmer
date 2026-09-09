//! AI agent that reads your RSS feeds and keeps only what's worth your time.

/// Error handler implementation.
pub mod error;

/// Main application.
pub mod app;

/// Command-line arguments parser.
pub mod args;

/// External Access.
pub mod ext;

/// State save in a file.
pub mod state;

/// Config file.
pub mod config;

/// Helper functions.
pub mod help;

/// Common types that can be glob-imported for convenience.
pub mod prelude;

use prelude::*;

use crate::app::App;

/// Runs `skimmer`.
///
/// # Errors
///
/// Returns an [`Error`] if the run fail.
pub async fn run(args: &Args) -> Result<()> {
    better_panic::install();

    let config = Config::load(args.config.as_deref())?;
    let state = State::load()?;

    let mut app = App::new(args, config, state).await?;
    app.run().await

    // TODO => Descide if keeping as aid
    // Debug related
    // if args.feeds {
    //     let titles = rss
    //         .get_feeds()
    //         .to_vec()
    //         .into_iter()
    //         .map(|feed| feed.title)
    //         .collect::<Vec<String>>();
    //     tracing::info!("{:#?}", titles);
    //     return Ok(());
    // }
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

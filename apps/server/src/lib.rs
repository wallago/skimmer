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

/// Web layer.
pub mod server;

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
    if let Some(Command::Serve) = args.command {
        // Runs until Ctrl-C or until the server stops.
        tokio::select! {
            result = tokio::signal::ctrl_c() => {
                tracing::info!("Interrupted: {:?}", result);
            },
            result = server::start(&config) => {
                tracing::info!("Server stopped: {:?}", result);
                result?;
            },
        }
        return Ok(());
    }
    let state = State::load(None)?;

    let mut app = App::new(args, config, state).await?;
    app.run().await

    // TODO => Descid if keeping as aid
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

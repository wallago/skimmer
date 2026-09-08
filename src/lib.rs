//! AI agent that reads your RSS feeds and keeps only what's worth your time.

/// Error handler implementation.
pub mod error;

/// Main application.
pub mod app;

/// Command-line arguments parser.
pub mod args;

/// Claude API.
pub mod claude;

/// Rss server.
pub mod rss;

/// Report.
pub mod report;

/// State save in a file.
pub mod state;

/// Topic to analyze.
pub mod topic;

/// Config file.
pub mod config;

/// Helper functions.
pub mod help;

/// Common types that can be glob-imported for convenience.
pub mod prelude;

use std::str::FromStr;

use miniflux_api::models::Entry;
use prelude::*;

use crate::{config::Config, rss::Rss, state::State};

/// Runs `skimmer`.
///
/// # Errors
///
/// Returns an [`Error`] if the run fa
pub async fn run(args: &Args) -> Result<()> {
    better_panic::install();

    // File related
    let config = Config::load(args.config.as_deref())?;
    let mut state = State::load()?;

    // Connection related
    let (url, username, password) = config.miniflux.get_all();
    let rss = Rss::new(url::Url::from_str(&url)?, username, password).await?;
    let claude = claude::Claude::new(config.claude);

    // Debug related
    if args.feeds {
        let titles = rss
            .get_feeds()
            .to_vec()
            .into_iter()
            .map(|feed| feed.title)
            .collect::<Vec<String>>();
        tracing::info!("{:#?}", titles);
        return Ok(());
    }

    // Generation related
    let topics = topic::Topic::new(config.topic, args.since, &state, &rss)?;
    let mut report = report::Report::new();

    for (_, topic) in topics {
        let mut entries: Vec<Entry> = Vec::new();
        for (_, feed) in topic.get_feeds() {
            entries.extend(
                rss.get_entries(feed.id, topic.get_last_run().timestamp())
                    .await?,
            );
        }

        let prompt = format!(
            "Here are the entries published since {}:\n\n{}\n\n{}",
            topic.get_last_run().to_rfc3339(),
            Rss::render_entries(&entries),
            topic.get_question(),
        );
        if args.dry_run {
            let tokens = claude.count_tokens(&prompt).await?;
            tracing::info!(
                "{tokens} input tokens ≈ ${:.4}",
                f64::from(tokens) * 5.0 / 1e6
            );
        } else {
            if entries.is_empty() {
                tracing::warn!(
                    "{}: no entries since {}",
                    topic.get_name(),
                    topic.get_last_run()
                );
                continue;
            }
            let resp = claude.request_something(&prompt).await?;
            report.add(&topic, &entries, resp);
        }
        // TODO -> Mark as read for rss
        // state.set_last_run(&topic)?;
    }
    report.generate(&claude)?;
    state.save()?;

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

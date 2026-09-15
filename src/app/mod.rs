//! Main application.

use std::{collections::HashMap, str::FromStr};

use crate::{
    app::{report::Report, topic::Topic},
    ext::prelude::{Claude, Entry, Rss},
    prelude::*,
};

/// Subject topic.
mod topic;

/// Report.
mod report;

/// Render output.
mod render;

/// Token counts for one call.
struct Tokens {
    /// Tokens billed at the input rate.
    input: u32,
    /// Tokens billed at the output rate.
    output: u32,
}

/// Everything a run needs, wired together once.
pub(crate) struct App {
    /// File save.
    state: State,
    /// Feed source.
    rss: Rss,
    /// Summarizer.
    claude: Claude,
    /// Topics to brief on, keyed by name.
    topics: HashMap<String, Topic>,
    /// Briefings collected so far.
    report: Report,
    /// Dry run to no token cost.
    dry_run: bool,
}

impl App {
    /// Connects to the RSS server and resolves the configured topics.
    pub(crate) async fn new(args: &Args, config: Config, state: State) -> Result<Self> {
        let (url_raw, username, password) = config.miniflux.get_all();
        let url = url::Url::from_str(&url_raw)?;
        let rss = Rss::new(url, username, password).await?;
        let claude = Claude::new(&config.claude);
        let topics = Topic::new(&config.topic, args.since, &state, &rss)?;
        let report = Report::new(config.output)?;
        Ok(Self {
            state,
            rss,
            claude,
            topics,
            report,
            dry_run: args.dry_run,
        })
    }

    /// Briefs every topic.
    pub(crate) async fn run(&mut self) -> Result<()> {
        let mut read: Vec<i64> = Vec::new();
        let mut total_usage = Tokens {
            input: 0,
            output: 0,
        };
        for (name, topic) in &self.topics {
            let (prompt, entries) = self.generate_prompt(topic).await?;
            let entry_ids = entries.iter().map(|entry| entry.id).collect::<Vec<i64>>();
            read.extend(&entry_ids);
            tracing::info!("\nEntries scanned: {}", entry_ids.len());
            if self.dry_run {
                let tokens = self.claude.count_tokens(&prompt).await?;
                self.claude.calculate_cost(tokens);
            } else {
                let (resp, usage) = self.claude.request_something(&prompt).await?;
                total_usage.input += usage.input_tokens;
                total_usage.output += usage.output_tokens;
                self.report.add(topic, &entries, resp);
                self.state.set_last_run(name)?;
            }
        }
        if !self.dry_run {
            tracing::info!("\nTotal entries scanned: {}", read.len());
            if let Some((in_rate, out_rate)) = self.claude.rates() {
                tracing::info!(
                    "Claude token usage:\n • Input  => {} ({}$)\n • Output => {} ({}$)",
                    total_usage.input,
                    (f64::from(total_usage.input) * in_rate) / 1e6,
                    total_usage.output,
                    (f64::from(total_usage.output) * out_rate) / 1e6,
                );
            } else {
                tracing::info!(
                    "Total token usage:\n • Input  => {}\n • Output => {}",
                    total_usage.input,
                    total_usage.output,
                );
            }
            self.report.generate(&self.claude)?;
            self.state.save()?;
            self.rss.mark_as_read(&read).await?;
        }
        Ok(())
    }

    /// Fetches the topic's entries since its last run and lays them out for Claude.
    async fn generate_prompt(&self, topic: &Topic) -> Result<(String, Vec<Entry>)> {
        let mut entries: Vec<Entry> = Vec::new();
        for feed in topic.get_feeds().values() {
            entries.extend(
                self.rss
                    .get_entries(feed.id, topic.get_last_run().timestamp())
                    .await?,
            );
        }

        if entries.is_empty() {
            tracing::warn!(
                "Topic {}: no entries since {}",
                topic.get_name(),
                topic.get_last_run()
            );
        }

        Ok((
            format!(
                "Here are the entries published since {}:\n\n{}\n\n{}",
                topic.get_last_run().to_rfc3339(),
                Rss::render_entries(&entries),
                topic.get_question(),
            ),
            entries,
        ))
    }
}

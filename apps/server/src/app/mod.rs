//! Main application.

use std::{collections::HashMap, str::FromStr};

use crate::{
    app::{
        render::Briefing,
        store::{Spend, Store},
        topic::Topic,
    },
    ext::prelude::{Claude, Entry, Rss, Usage},
    prelude::*,
};

/// Subject topic.
mod topic;

/// One topic's briefing.
mod render;

/// Briefings database.
pub(crate) mod store;

/// What one web search costs, on top of the tokens its results add.
const SEARCH_USD: f64 = 0.01;

/// Token counts for one call.
struct Tokens {
    /// Tokens billed at the input rate.
    input: u32,
    /// Tokens billed at the output rate.
    output: u32,
}

/// Everything a run needs, wired together once.
pub(crate) struct App {
    /// Feed source.
    rss: Rss,
    /// Summarizer.
    claude: Claude,
    /// Topics to brief on, keyed by name.
    topics: HashMap<String, Topic>,
    /// Where briefings are saved for the web app.
    store: Store,
    /// Dry run to no token cost.
    dry_run: bool,
}

impl App {
    /// Connects to the RSS server and resolves the configured topics.
    pub(crate) async fn new(args: &Args, config: Config) -> Result<Self> {
        let (url_raw, username, password) = config.miniflux.get_all();
        let url = url::Url::from_str(url_raw)?;
        let rss = Rss::new(url, username, password).await?;
        let claude = Claude::new(&config);
        let store = Store::open(config.database)?;
        let topics = Topic::new(&config.topic, args.since, &store, &rss)?;
        Ok(Self {
            rss,
            claude,
            topics,
            store,
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
            let (prompt, entries, searched) = self.generate_prompt(topic).await?;
            if entries.is_empty() {
                continue;
            }
            // Web results carry negative ids and are not Miniflux entries.
            read.extend(entries.iter().map(|entry| entry.id).filter(|id| *id > 0));
            tracing::info!("\nEntries scanned: {}", entries.len());
            if self.dry_run {
                let tokens = self.claude.count_tokens(&prompt).await?;
                self.claude.calculate_cost(tokens);
            } else {
                let (resp, usage) = self.claude.request_something(&prompt).await?;
                let input = usage.input_tokens + searched.input_tokens;
                let output = usage.output_tokens + searched.output_tokens;
                total_usage.input += input;
                total_usage.output += output;
                let briefing = Briefing::new(topic, &entries, resp);
                let spend = Spend {
                    input_tokens: input,
                    output_tokens: output,
                    cost_usd: self.claude.rates().map(|(in_rate, out_rate)| {
                        (f64::from(input) * in_rate + f64::from(output) * out_rate) / 1e6
                            + f64::from(searched.server_tool_use.web_search_requests) * SEARCH_USD
                    }),
                };
                self.store.add(&briefing, self.claude.get_model(), spend)?;
                self.store.set_last_run(name)?;
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
            self.rss.mark_as_read(&read).await?;
        }
        Ok(())
    }

    /// Fetches the topic's entries since its last run, plus what a web
    /// search finds when the topic asks for one, and lays them out for
    /// Claude. Also returns what the search cost.
    async fn generate_prompt(&self, topic: &Topic) -> Result<(String, Vec<Entry>, Usage)> {
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
                "Topic {}: no entries since {} in its {} feed(s): {:?}",
                topic.get_name(),
                topic.get_last_run(),
                topic.get_feeds().len(),
                topic.get_feeds().keys().collect::<Vec<_>>(),
            );
        }

        let ask = if topic.get_context().trim().is_empty() {
            topic.get_question().to_owned()
        } else {
            format!(
                "{}\n\nWhat matters to the reader on this topic:\n{}",
                topic.get_question(),
                topic.get_context(),
            )
        };

        let mut searched = Usage::default();
        if let Some(web) = topic.get_web_search() {
            if self.dry_run {
                tracing::info!(
                    "Topic {}: would search the web up to {} time(s), ${:.2} at most",
                    topic.get_name(),
                    web.max_uses,
                    f64::from(web.max_uses) * SEARCH_USD,
                );
            } else {
                let (found, usage) = self
                    .claude
                    .search_web(&ask, topic.get_last_run(), web)
                    .await?;
                tracing::info!(
                    "Topic {}: {} web search(es) found {} page(s)",
                    topic.get_name(),
                    usage.server_tool_use.web_search_requests,
                    found.len(),
                );
                entries.extend(found);
                searched = usage;
            }
        }

        Ok((
            format!(
                "Here are the entries published since {}:\n\n{}\n\n{ask}",
                topic.get_last_run().to_rfc3339(),
                Rss::render_entries(&entries),
            ),
            entries,
            searched,
        ))
    }
}

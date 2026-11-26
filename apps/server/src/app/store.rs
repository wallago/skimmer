//! The briefings database (SQLite): one row per topic per run, with its read
//! state, so the web app can list days and count what is still unread.

use std::path::PathBuf;

use chrono::{Local, NaiveDate, Utc};
use dirs::state_dir;
use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::{Deserialize, Serialize};

use crate::{app::render::Briefing, prelude::*};

/// Schema steps, applied in order. `PRAGMA user_version` records how many ran.
const MIGRATIONS: &[&str] = &[
    "
create table briefings (
    id          integer primary key,
    topic       text not null,
    day         text not null,  -- local date the briefing belongs to, YYYY-MM-DD
    question    text not null,
    since       text not null,  -- start of the window, RFC 3339
    scanned     integer not null,
    digest      text not null,
    highlights  text not null,  -- JSON: [{headline, detail, entry_ids}]
    sources     text not null,  -- JSON: [{id, title, feed, url}]
    model       text not null,
    status      text not null default 'unread'
                check (status in ('unread', 'viewed', 'finished')),
    created_at  text not null,
    updated_at  text not null
);
create index briefings_day on briefings (day);
",
    "
alter table briefings add column input_tokens  integer not null default 0;
alter table briefings add column output_tokens integer not null default 0;
alter table briefings add column cost_usd      real;  -- null: model not in the price table
",
];

/// How far the reader got through a briefing.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Status {
    /// Not opened yet.
    Unread,
    /// Opened, not read to the end.
    Viewed,
    /// Read to the end, or marked read.
    Finished,
}

impl Status {
    /// The value stored in `briefings.status`.
    fn as_str(self) -> &'static str {
        match self {
            Self::Unread => "unread",
            Self::Viewed => "viewed",
            Self::Finished => "finished",
        }
    }
}

/// What one Claude call used, and what it cost.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Spend {
    /// Tokens billed at the input rate.
    pub(crate) input_tokens: u32,
    /// Tokens billed at the output rate.
    pub(crate) output_tokens: u32,
    /// USD, or [`None`] for a model without a known price.
    pub(crate) cost_usd: Option<f64>,
}

/// One day that has briefings: how many are in each state, and what they
/// cost to write.
#[derive(Debug, Serialize, PartialEq)]
pub(crate) struct DayCount {
    /// Local date, `YYYY-MM-DD`.
    pub(crate) day: String,
    /// Not opened yet.
    pub(crate) unread: i64,
    /// Opened, not finished.
    pub(crate) viewed: i64,
    /// Finished.
    pub(crate) finished: i64,
    /// Input tokens used that day.
    pub(crate) input_tokens: i64,
    /// Output tokens used that day.
    pub(crate) output_tokens: i64,
    /// USD spent that day, [`None`] if no briefing had a known price.
    pub(crate) cost_usd: Option<f64>,
}

/// A briefing as saved, for the web app.
#[derive(Debug, Serialize)]
pub(crate) struct Saved {
    /// Row id.
    pub(crate) id: i64,
    /// Topic name.
    pub(crate) topic: String,
    /// Local date, `YYYY-MM-DD`.
    pub(crate) day: String,
    /// The question put to Claude.
    pub(crate) question: String,
    /// Start of the window, RFC 3339.
    pub(crate) since: String,
    /// How many entries were scanned.
    pub(crate) scanned: i64,
    /// Claude's prose synthesis.
    pub(crate) digest: String,
    /// `[{headline, detail, entry_ids}]`.
    pub(crate) highlights: serde_json::Value,
    /// `[{id, title, feed, url}]`.
    pub(crate) sources: serde_json::Value,
    /// Model that wrote it.
    pub(crate) model: String,
    /// Read state.
    pub(crate) status: Status,
    /// When the read state last changed, RFC 3339.
    pub(crate) updated_at: String,
}

impl Saved {
    /// Reads one `select *`-shaped row.
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        let json = |index: usize| -> rusqlite::Result<serde_json::Value> {
            let raw: String = row.get(index)?;
            serde_json::from_str(&raw).map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    index,
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })
        };
        let status: String = row.get("status")?;
        Ok(Self {
            id: row.get("id")?,
            topic: row.get("topic")?,
            day: row.get("day")?,
            question: row.get("question")?,
            since: row.get("since")?,
            scanned: row.get("scanned")?,
            digest: row.get("digest")?,
            highlights: json(7)?,
            sources: json(8)?,
            model: row.get("model")?,
            status: match status.as_str() {
                "viewed" => Status::Viewed,
                "finished" => Status::Finished,
                _ => Status::Unread,
            },
            updated_at: row.get("updated_at")?,
        })
    }
}

/// The open database.
pub(crate) struct Store {
    /// The database connection.
    conn: Connection,
}

impl Store {
    /// Opens the database at `path`, else `$XDG_STATE_HOME/skimmer/skimmer.db`,
    /// creating and migrating it as needed.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if there is no state directory, or the database
    /// cannot be opened or migrated.
    pub(crate) fn open(path: Option<PathBuf>) -> Result<Self> {
        let path = match path {
            Some(path) => path,
            None => state_dir()
                .ok_or(Error::StatePath)?
                .join(env!("CARGO_PKG_NAME"))
                .join("skimmer.db"),
        };
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        Self::migrate(Connection::open(path)?)
    }

    /// An in-memory store holding one sample briefing on `day`, for tests.
    #[cfg(test)]
    pub(crate) fn sample(day: NaiveDate) -> Self {
        let store = Self::migrate(Connection::open_in_memory().unwrap()).unwrap();
        store
            .add_on(day, &tests::briefing(), "m", Spend::default())
            .unwrap();
        store
    }

    /// Brings the schema up to date.
    fn migrate(conn: Connection) -> Result<Self> {
        let done: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        for (version, sql) in (1_i64..).zip(MIGRATIONS) {
            if version > done {
                conn.execute_batch(sql)?;
                conn.pragma_update(None, "user_version", version)?;
            }
        }
        Ok(Self { conn })
    }

    /// Saves one topic's briefing as unread, under today's local date.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if serializing or inserting fails.
    pub(crate) fn add(&self, briefing: &Briefing, model: &str, spend: Spend) -> Result<()> {
        self.add_on(Local::now().date_naive(), briefing, model, spend)
    }

    /// Every day with briefings, newest first.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the query fails.
    pub(crate) fn days(&self) -> Result<Vec<DayCount>> {
        let mut statement = self.conn.prepare(
            "select day,
                    count(*) filter (where status = 'unread'),
                    count(*) filter (where status = 'viewed'),
                    count(*) filter (where status = 'finished'),
                    sum(input_tokens), sum(output_tokens), sum(cost_usd)
            from briefings group by day order by day desc",
        )?;
        let days = statement.query_map([], |row| {
            Ok(DayCount {
                day: row.get(0)?,
                unread: row.get(1)?,
                viewed: row.get(2)?,
                finished: row.get(3)?,
                input_tokens: row.get(4)?,
                output_tokens: row.get(5)?,
                cost_usd: row.get(6)?,
            })
        })?;
        Ok(days.collect::<rusqlite::Result<_>>()?)
    }

    /// The briefings of one `day`, in the order they were saved.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the query fails.
    pub(crate) fn on_day(&self, day: NaiveDate) -> Result<Vec<Saved>> {
        let mut statement = self
            .conn
            .prepare("select * from briefings where day = ?1 order by id")?;
        let saved = statement.query_map([day.to_string()], Saved::from_row)?;
        Ok(saved.collect::<rusqlite::Result<_>>()?)
    }

    /// Sets a briefing's read state. [`None`] if there is no briefing `id`.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the update fails.
    pub(crate) fn set_status(&self, id: i64, status: Status) -> Result<Option<Saved>> {
        self.conn.execute(
            "update briefings set status = ?2, updated_at = ?3 where id = ?1",
            params![id, status.as_str(), Utc::now().to_rfc3339()],
        )?;
        Ok(self
            .conn
            .query_row(
                "select * from briefings where id = ?1",
                [id],
                Saved::from_row,
            )
            .optional()?)
    }

    /// [`add`](Self::add), for a given `day`.
    fn add_on(&self, day: NaiveDate, briefing: &Briefing, model: &str, spend: Spend) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "insert into briefings (topic, day, question, since, scanned, digest, highlights,
                                    sources, model, created_at, updated_at,
                                    input_tokens, output_tokens, cost_usd)
            values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10, ?11, ?12, ?13)",
            params![
                briefing.topic,
                day.to_string(),
                briefing.question,
                briefing.since.to_rfc3339(),
                i64::try_from(briefing.scanned).unwrap_or(i64::MAX),
                briefing.analysis.get_digest(),
                serde_json::to_string(briefing.analysis.get_highlights())?,
                serde_json::to_string(&briefing.sources)?,
                model,
                now,
                spend.input_tokens,
                spend.output_tokens,
                spend.cost_usd,
            ],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::app::render::Source;

    /// A briefing with one highlight citing its one source.
    pub(super) fn briefing() -> Briefing {
        Briefing {
            topic: "rust".to_string(),
            question: "What's new?".to_string(),
            since: "2026-11-22T06:00:00Z".parse().unwrap(),
            analysis: serde_json::from_str(
                r#"{"digest": "A quiet day.",
                    "highlights": [{"headline": "H", "detail": "D", "entry_ids": [42]}]}"#,
            )
            .unwrap(),
            sources: vec![Source {
                id: 42,
                title: "T".to_string(),
                feed: "F".to_string(),
                url: "https://example.com/42".to_string(),
            }],
            scanned: 1,
        }
    }

    /// A fresh in-memory store.
    fn store() -> Store {
        Store::migrate(Connection::open_in_memory().unwrap()).unwrap()
    }

    #[test]
    fn days_count_each_state_and_sum_spend_newest_first() {
        let store = store();
        let monday = NaiveDate::from_ymd_opt(2026, 11, 23).unwrap();
        let sunday = NaiveDate::from_ymd_opt(2026, 11, 22).unwrap();
        let priced = Spend {
            input_tokens: 1000,
            output_tokens: 200,
            cost_usd: Some(0.25),
        };
        let unpriced = Spend {
            cost_usd: None,
            ..priced
        };
        store.add_on(sunday, &briefing(), "m", unpriced).unwrap();
        for _ in 0..3 {
            store.add_on(monday, &briefing(), "m", priced).unwrap();
        }
        store.set_status(2, Status::Viewed).unwrap();
        store.set_status(3, Status::Finished).unwrap();
        assert_eq!(
            store.days().unwrap(),
            vec![
                DayCount {
                    day: "2026-11-23".to_string(),
                    unread: 1,
                    viewed: 1,
                    finished: 1,
                    input_tokens: 3000,
                    output_tokens: 600,
                    cost_usd: Some(0.75),
                },
                DayCount {
                    day: "2026-11-22".to_string(),
                    unread: 1,
                    viewed: 0,
                    finished: 0,
                    input_tokens: 1000,
                    output_tokens: 200,
                    cost_usd: None,
                },
            ]
        );
    }

    #[test]
    fn on_day_reads_back_what_was_saved() {
        let store = store();
        let day = NaiveDate::from_ymd_opt(2026, 11, 23).unwrap();
        store
            .add_on(day, &briefing(), "claude-haiku-5-5", Spend::default())
            .unwrap();
        let saved = store.on_day(day).unwrap();
        assert_eq!(saved.len(), 1);
        let first = &saved[0];
        assert_eq!(first.topic, "rust");
        assert_eq!(first.status, Status::Unread);
        assert_eq!(first.highlights[0]["entry_ids"][0], 42);
        assert_eq!(first.sources[0]["url"], "https://example.com/42");
        assert!(store.on_day(day.succ_opt().unwrap()).unwrap().is_empty());
    }

    #[test]
    fn set_status_answers_the_updated_briefing_or_none() {
        let store = store();
        let day = NaiveDate::from_ymd_opt(2026, 11, 23).unwrap();
        store
            .add_on(day, &briefing(), "m", Spend::default())
            .unwrap();
        let saved = store.set_status(1, Status::Finished).unwrap().unwrap();
        assert_eq!(saved.status, Status::Finished);
        assert!(store.set_status(99, Status::Viewed).unwrap().is_none());
    }

    #[test]
    fn migrate_twice_is_a_no_op() {
        let store = Store::migrate(Connection::open_in_memory().unwrap()).unwrap();
        let store = Store::migrate(store.conn).unwrap();
        let version: i64 = store
            .conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, i64::try_from(MIGRATIONS.len()).unwrap());
    }

    #[test]
    fn add_saves_an_unread_briefing() {
        let store = Store::migrate(Connection::open_in_memory().unwrap()).unwrap();
        let day = NaiveDate::from_ymd_opt(2026, 11, 23).unwrap();
        store
            .add_on(day, &briefing(), "claude-haiku-5-5", Spend::default())
            .unwrap();

        let row: (String, String, String, String, String, String) = store
            .conn
            .query_row(
                "select topic, day, status, digest, highlights, sources from briefings",
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                    ))
                },
            )
            .unwrap();
        assert_eq!(
            row,
            (
                "rust".to_string(),
                "2026-11-23".to_string(),
                "unread".to_string(),
                "A quiet day.".to_string(),
                r#"[{"headline":"H","detail":"D","entry_ids":[42]}]"#.to_string(),
                r#"[{"id":42,"title":"T","feed":"F","url":"https://example.com/42"}]"#.to_string(),
            )
        );
    }
}

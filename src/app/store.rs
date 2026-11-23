//! The briefings database (SQLite): one row per topic per run, with its read
//! state, so the web app can list days and count what is still unread.

use std::path::PathBuf;

use chrono::{Local, NaiveDate, Utc};
use dirs::state_dir;
use rusqlite::{Connection, params};

use crate::{app::render::Briefing, prelude::*};

/// Schema steps, applied in order. `PRAGMA user_version` records how many ran.
const MIGRATIONS: &[&str] = &["
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
"];

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
    pub(crate) fn add(&self, briefing: &Briefing, model: &str) -> Result<()> {
        self.add_on(Local::now().date_naive(), briefing, model)
    }

    /// [`add`](Self::add), for a given `day`.
    fn add_on(&self, day: NaiveDate, briefing: &Briefing, model: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "insert into briefings (topic, day, question, since, scanned, digest, highlights,
                                    sources, model, created_at, updated_at)
             values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
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
    fn briefing() -> Briefing {
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
        store.add_on(day, &briefing(), "claude-haiku-5-5").unwrap();

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

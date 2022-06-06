use std::{time::{SystemTime, UNIX_EPOCH}, path::Path};

use r2d2::Pool;
use r2d2_sqlite::{rusqlite::params, SqliteConnectionManager};

use anyhow::Result;
use toro::Toro;

use crate::event::EventVersion;

pub struct Db {
    pub pool: Pool<SqliteConnectionManager>,
}

pub struct EventRow {
    pub version: u64,
    pub event_toro: String,
    pub timestamp: u64,
}

impl Db {
    pub fn init(filename: impl AsRef<Path>) -> Result<Self> {
        let manager = SqliteConnectionManager::file(filename);
        let s = Self {
            pool: Pool::new(manager)?,
        };
        s.init_table()?;
        Ok(s)
    }

    fn init_table(&self) -> Result<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS events (
                    version INTEGER PRIMARY KEY,
                    event_toro STRING NOT NULL,
                    timestamp INTEGER NOT NULL
                );",
            params![],
        )?;
        Ok(())
    }

    pub fn insert_event(&self, toro: Toro) -> Result<()> {
        let conn = self.pool.get()?;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backward.")
            .as_secs();
        conn.execute(
            "INSERT INTO events
            (event_toro, timestamp)
            VALUES
            (?1, ?2);
            ",
            params![toro.to_toro_string(), timestamp],
        )?;
        Ok(())
    }

    pub fn get_events(&self, from_version: EventVersion) -> Result<Vec<EventRow>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "
            SELECT version, event_toro, timestamp
            FROM events
            WHERE version >= ?1;
            ",
        )?;
        let rows = stmt
            .query_map(params![from_version], |row| {
                Ok(EventRow {
                    version: row.get(0)?,
                    event_toro: row.get(1)?,
                    timestamp: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<EventRow>, _>>();
        Ok(rows?)
    }
}

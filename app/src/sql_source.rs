use std::sync::Arc;

use anyhow::Result;
use toro::Toro;

use crate::{db::Db, event::Event, event::EventVersion, projector::EventSource};

pub struct SqliteEventSource {
    db: Arc<Db>,
}

impl SqliteEventSource {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }
}

impl EventSource for SqliteEventSource {
    fn fetch(&self, from_version: EventVersion) -> Result<Vec<Event>> {
        let rows = self.db.get_events(from_version)?;
        let toros = rows
            .iter()
            .map(|r| Toro::from_toro_string(&r.event_toro))
            .collect::<Result<Vec<Toro>>>()?;
        rows.iter()
            .zip(toros.iter())
            .map(|(r, t)| Event::from_toro(t, r.version, r.timestamp))
            .collect()
    }
}

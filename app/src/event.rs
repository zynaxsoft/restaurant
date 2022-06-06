use crate::restaurant::{Item, TableId, Time};

use anyhow::{anyhow, bail, Result};

// Starts from 1
pub type EventVersion = u64;

pub use toro::Command;
use toro::{Toro, Param};

#[derive(Debug, Clone)]
pub struct Event {
    pub version: EventVersion,
    pub command: Command,
    pub payload: Option<Payload>,
    pub created: Time,
}

impl Event {
    pub fn new(version: EventVersion, command: Command, time: Time) -> Self {
        Event {
            version,
            command,
            payload: None,
            created: time,
        }
    }

    pub fn from_toro(toro: &Toro, version: EventVersion, timestamp: Time) -> Result<Self> {
        let command = toro.command;
        let table_id = toro.table_id.ok_or(anyhow!("Expecting table id"))?;
        let payload = match command {
            Command::Yeet => None,
            Command::New | Command::Cancel => {
                let items = match &toro.param {
                    Some(Param::MenuQuantities(v)) => {
                        v.iter().map(|mq| Item::new(mq.0.clone(), mq.1, timestamp)).collect()
                    },
                    _ => bail!("This Toro doesn't make sense: {}", toro.to_toro_string())
                };
                Some(Payload::new(table_id, items))
            },
            _ => bail!(
                "Command {:?} is not in the event spec. How did you get this?",
                command
            ),
        };
        Ok(Event {
            version,
            command,
            payload,
            created: timestamp,
        })
    }

    pub fn with_payload(mut self, payload: Payload) -> Self {
        self.payload = Some(payload);
        self
    }
}

#[derive(Debug, Clone)]
pub struct Payload {
    pub table_id: TableId,
    pub items: Vec<Item>,
}

impl Payload {
    pub fn new(table_id: TableId, items: Vec<Item>) -> Self {
        Self { table_id, items }
    }
}

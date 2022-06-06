use crate::{
    event::{Command, Event, EventVersion, Payload},
    restaurant::{CookingTimeEstimator, Table, TableId, Item},
};

use anyhow::{bail, Result};
use tracing::instrument;

pub trait EventSource {
    // Get events starting from `from_version` until the latest event
    fn fetch(&self, from_version: EventVersion) -> Result<Vec<Event>>;
}

pub struct RestaurantProjector<T> {
    pub current_version: EventVersion,
    pub tables: Vec<Table>,
    source: T,
}

impl<T> RestaurantProjector<T>
where
    T: EventSource,
{
    pub fn new(n_table: u64, source: T) -> Self {
        // In this restaurant table number begins with 0!
        let tables: Vec<Table> = (0..n_table as usize).map(|n| Table::new(n)).collect();
        Self {
            current_version: 0,
            tables,
            source,
        }
    }

    #[instrument(level = "debug", name = "Updating projector", skip(self))]
    pub fn update(&mut self) -> Result<()> {
        for event in self.source.fetch(self.current_version + 1)? {
            self.project(event)?;
            self.current_version += 1;
        }
        Ok(())
    }

    pub fn get_table(&self, id: TableId) -> Option<&Table> {
        self.tables.get(id)
    }

    fn process_new_cmd(&mut self, payload: Payload) -> Result<()> {
        if let Some(table) = self.tables.get_mut(payload.table_id) {
            for i in payload.items.into_iter() {
                let new_item = match table.items.get(&i.id) {
                    Some(t) => {
                        let new_quant = t.quantity + i.quantity;
                        Item::new(i.id, new_quant, i.timestamp)
                    }
                    None => {
                        Item::new(i.id, i.quantity, i.timestamp)
                    }
                };
                let cooking_time =
                    CookingTimeEstimator::estimate(&new_item, self.current_version);
                let new_item = new_item.with_cooking_time(cooking_time);
                table.items.insert(new_item.id.clone(), new_item);
            }
            return Ok(());
        }
        bail!("Table {} doesn't exist!", payload.table_id)
    }

    fn process_cancel(&mut self, payload: Payload) -> Result<()> {
        if let Some(table) = self.tables.get_mut(payload.table_id) {
            for item in payload.items.iter() {
                if let Some(target_item) = table.items.get_mut(&item.id) {
                    let quantity = target_item.quantity - item.quantity;
                    if quantity <= 0 {
                        table.items.remove(&item.id);
                    } else {
                        target_item.quantity = quantity;
                    }
                }
            }
            return Ok(());
        }
        bail!("Table {} doesn't exist!", payload.table_id)
    }

    fn process_yeet(&mut self) -> Result<()> {
        for table in self.tables.iter_mut() {
            table.reset();
        }
        Ok(())
    }

    #[instrument(level = "debug", name = "Projecting event", skip(self))]
    fn project(&mut self, event: Event) -> Result<()> {
        // In this projector, we only care new order, cancel, and yeet events.
        match event.command {
            Command::New => match event.payload {
                Some(payload) => self.process_new_cmd(payload)?,
                None => bail!("No payload available"),
            },
            Command::Cancel => match event.payload {
                Some(payload) => self.process_cancel(payload)?,
                None => bail!("No payload available"),
            },
            Command::Yeet => self.process_yeet()?,
            _ => (),
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    mod test_helper {
        use crate::{
            event::{Command, Payload},
            restaurant::Item,
        };

        use super::*;
        pub(super) struct MyEventSource {
            max_version: EventVersion,
            events: Vec<Event>,
        }

        impl MyEventSource {
            pub(super) fn new(max_version: EventVersion) -> Self {
                Self {
                    max_version,
                    events: vec![
                        Event::new(1, Command::New, 1)
                            .with_payload(Payload::new(0, vec![Item::new("a".into(), 1, 1)])),
                        Event::new(2, Command::New, 1)
                            .with_payload(Payload::new(1, vec![Item::new("b".into(), 2, 1)])),
                        Event::new(3, Command::Cancel, 1)
                            .with_payload(Payload::new(0, vec![Item::new("a".into(), 1, 1)])),
                        Event::new(4, Command::Cancel, 1)
                            .with_payload(Payload::new(1, vec![Item::new("b".into(), 1, 1)])),
                        Event::new(5, Command::Cancel, 1)
                            .with_payload(Payload::new(1, vec![Item::new("b".into(), 1, 1)])),
                        Event::new(6, Command::New, 1)
                            .with_payload(Payload::new(0, vec![Item::new("a".into(), 1, 1)])),
                        Event::new(7, Command::New, 1)
                            .with_payload(Payload::new(1, vec![Item::new("b".into(), 2, 1)])),
                        Event::new(8, Command::Yeet, 1),
                    ],
                }
            }
        }

        impl EventSource for MyEventSource {
            fn fetch(&self, from_version: EventVersion) -> Result<Vec<Event>> {
                let result = self
                    .events
                    .iter()
                    .filter(|e| e.version >= from_version && e.version <= self.max_version)
                    .cloned()
                    .collect();
                Ok(result)
            }
        }

        pub(super) fn initialize_projector_to_version(
            ver: EventVersion,
        ) -> RestaurantProjector<MyEventSource> {
            let source = MyEventSource::new(ver);
            let mut projector = RestaurantProjector::new(10, source);
            projector.update().unwrap();
            projector
        }
    }
    use self::test_helper::initialize_projector_to_version;

    use super::*;

    #[test]
    fn test_new_order_projection() {
        let projector = initialize_projector_to_version(2);
        let item1 = projector.tables[0].items.get("a").unwrap();
        assert_eq!(item1.quantity, 1);
        let item2 = projector.tables[1].items.get("b").unwrap();
        assert_eq!(item2.quantity, 2);
    }

    #[test]
    fn test_cancel_order_projection() {
        let projector = initialize_projector_to_version(4);
        assert!(projector.tables[0].items.get("a").is_none());
        assert!(projector.tables[1].items.get("b").is_some());
        let projector = initialize_projector_to_version(5);
        assert!(projector.tables[0].items.get("a").is_none());
        assert!(projector.tables[1].items.get("b").is_none());
    }

    #[test]
    fn test_yeet_projection() {
        let projector = initialize_projector_to_version(8);
        assert!(projector.tables[0].items.get("a").is_none());
        assert!(projector.tables[1].items.get("b").is_none());
    }
}

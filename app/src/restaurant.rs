use std::collections::HashMap;

use crate::event::EventVersion;

pub type TableId = usize;
pub type ItemId = String;
pub type Quantity = i64;
// Time represent a time interval with a unit of seconds
pub type Time = u64;
pub type Menu = String;

#[derive(Debug)]
pub struct Table {
    pub id: TableId,
    pub items: HashMap<ItemId, Item>,
}

impl Table {
    pub fn new(id: TableId) -> Self {
        Table {
            id,
            items: HashMap::default(),
        }
    }

    pub fn reset(&mut self) {
        self.items = HashMap::default();
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    pub id: ItemId,
    pub quantity: Quantity,
    pub cooking_time: Option<Time>,
    pub timestamp: Time,
}

impl Item {
    pub fn new(id: ItemId, quantity: Quantity, timestamp: Time) -> Self {
        Self {
            id,
            quantity,
            cooking_time: None,
            timestamp,
        }
    }

    pub fn with_cooking_time(mut self, time: Time) -> Self {
        self.cooking_time = Some(time);
        self
    }
}

// This serves as an abstract purpose
// It should provide an accurate cooking time based on the available data.
// Such as Kitchen workload, orders in queue and stuff.
pub struct CookingTimeEstimator;
impl CookingTimeEstimator {
    pub fn estimate(item: &Item, event_version: EventVersion) -> Time {
        // A very accurate estimation of a restaurant
        (item.id.len() as u64 * 60 + (event_version % 10) * 60)
            * ((item.quantity + 2) as f32).log(2.71) as u64
    }
}

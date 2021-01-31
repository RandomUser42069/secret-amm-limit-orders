use cosmwasm_std::{CanonicalAddr, Uint128};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering};
use schemars::JsonSchema;
use std::collections::{BinaryHeap};

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct OrderIndex {
    pub id: CanonicalAddr,
    pub price: Uint128,
    pub timestamp: u64
}

// Arrange at first by price and after that by timestamp
impl Ord for OrderIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.price < other.price {
            Ordering::Less
        } else if self.price > other.price {
            Ordering::Greater
        } else {
            // FIFO
            other.timestamp.cmp(&self.timestamp)
        }
    }
}

impl PartialOrd for OrderIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for OrderIndex {
    fn eq(&self, other: &Self) -> bool {
        if self.price > other.price || self.price < other.price {
            false
        } else {
            self.timestamp == other.timestamp
        }
    }
}

impl Eq for OrderIndex {}
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct OrderQueue {
    idx_queue: Option<BinaryHeap<OrderIndex>>
}

impl OrderQueue {
    pub fn new() -> Self {
        OrderQueue {
            idx_queue: Some(BinaryHeap::new())
        }
    }

    pub fn insert(&mut self, id: CanonicalAddr, price: Uint128, timestamp:u64 ) -> bool {
        self.idx_queue.as_mut().unwrap().push(OrderIndex {
            id,
            price,
            timestamp
        });
        true
    }

    pub fn peek(&mut self) -> Option<&OrderIndex> {
        self.idx_queue.as_mut().unwrap().peek()
    }

    pub fn pop(&mut self) -> Option<OrderIndex> {
        self.idx_queue.as_mut().unwrap().pop()
    }

    pub fn remove(&mut self, id: CanonicalAddr) {
        if let Some(idx_queue) = self.idx_queue.take() {
            let mut active_orders = idx_queue.into_vec();
            active_orders.retain(|order_id| id != order_id.id);
            self.idx_queue = Some(BinaryHeap::from(active_orders));
        }
    }
}
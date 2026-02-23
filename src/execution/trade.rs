use std::fmt;

use serde::{Deserialize, Serialize};

/// A trade that was made during a match
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trade {
    /// The ID of the maker order
    maker_order_id: u64,
    /// The price of the trade
    price: u64,
    /// The quantity of the trade
    quantity: u64,
}

impl Trade {
    /// Create a new trade
    pub fn new(maker_order_id: u64, price: u64, quantity: u64) -> Self {
        Self {
            maker_order_id,
            price,
            quantity,
        }
    }

    /// Get the ID of the maker order
    pub fn maker_order_id(&self) -> u64 {
        self.maker_order_id
    }

    /// Get the price of the trade
    pub fn price(&self) -> u64 {
        self.price
    }

    /// Get the quantity of the trade
    pub fn quantity(&self) -> u64 {
        self.quantity
    }
}

impl fmt::Display for Trade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Trade: maker_order_id={} price={} quantity={}",
            self.maker_order_id(),
            self.price(),
            self.quantity()
        )
    }
}

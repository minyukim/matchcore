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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_trade() -> Trade {
        Trade::new(1, 100, 10)
    }

    #[test]
    fn test_maker_order_id() {
        assert_eq!(create_trade().maker_order_id(), 1);
    }

    #[test]
    fn test_price() {
        assert_eq!(create_trade().price(), 100);
    }

    #[test]
    fn test_quantity() {
        assert_eq!(create_trade().quantity(), 10);
    }

    #[test]
    fn test_round_trip_serialization() {
        let trade = create_trade();
        let serialized = serde_json::to_string(&trade).unwrap();
        let deserialized: Trade = serde_json::from_str(&serialized).unwrap();
        assert_eq!(trade, deserialized);
    }

    #[test]
    fn test_display() {
        assert_eq!(
            create_trade().to_string(),
            "Trade: maker_order_id=1 price=100 quantity=10"
        );
    }
}

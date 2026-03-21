use crate::{OrderId, Price, Quantity};

use std::fmt;

/// A trade that was made during a match
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Trade {
    /// The ID of the maker order
    maker_order_id: OrderId,
    /// The price of the trade
    price: Price,
    /// The quantity of the trade
    quantity: Quantity,
}

impl Trade {
    /// Create a new trade
    pub(crate) fn new(maker_order_id: OrderId, price: Price, quantity: Quantity) -> Self {
        Self {
            maker_order_id,
            price,
            quantity,
        }
    }

    /// Get the ID of the maker order
    pub fn maker_order_id(&self) -> OrderId {
        self.maker_order_id
    }

    /// Get the price of the trade
    pub fn price(&self) -> Price {
        self.price
    }

    /// Get the quantity of the trade
    pub fn quantity(&self) -> Quantity {
        self.quantity
    }
}

impl fmt::Display for Trade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "maker({}): {}@{}",
            self.maker_order_id(),
            self.quantity(),
            self.price(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_trade() -> Trade {
        Trade::new(OrderId(1), Price(100), Quantity(10))
    }

    #[test]
    fn test_maker_order_id() {
        assert_eq!(create_trade().maker_order_id(), OrderId(1));
    }

    #[test]
    fn test_price() {
        assert_eq!(create_trade().price(), Price(100));
    }

    #[test]
    fn test_quantity() {
        assert_eq!(create_trade().quantity(), Quantity(10));
    }

    #[test]
    fn test_display() {
        assert_eq!(create_trade().to_string(), "maker(1): 10@100");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_round_trip_serialization() {
        let trade = create_trade();
        let serialized = serde_json::to_string(&trade).unwrap();
        let deserialized: Trade = serde_json::from_str(&serialized).unwrap();
        assert_eq!(trade, deserialized);
    }
}

use crate::{OrderId, Price, Quantity, QuantityPolicy, orders::OrderFlags};

use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

/// Generic limit order with various configuration options
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LimitOrder {
    /// The ID of the order
    id: OrderId,
    /// The specification of the order
    spec: LimitOrderSpec,
}

impl LimitOrder {
    /// Create a new limit order
    pub fn new(id: OrderId, spec: LimitOrderSpec) -> Self {
        Self { id, spec }
    }

    /// Get the order ID
    pub fn id(&self) -> OrderId {
        self.id
    }

    /// Get the specification of the order
    pub fn spec(&self) -> &LimitOrderSpec {
        &self.spec
    }

    /// Matches this order against an incoming quantity
    ///
    /// Returns a tuple containing:
    /// - The quantity consumed from the incoming order
    /// - The quantity that was replenished (for iceberg orders)
    pub(crate) fn match_against(&mut self, incoming_quantity: Quantity) -> (Quantity, Quantity) {
        match self.quantity_policy {
            QuantityPolicy::Standard { quantity } => {
                let new_quantity = quantity.saturating_sub(incoming_quantity);
                let consumed = quantity - new_quantity;

                self.quantity_policy.update_visible_quantity(new_quantity);
                (consumed, Quantity(0))
            }
            QuantityPolicy::Iceberg {
                visible_quantity, ..
            } => {
                let new_visible = visible_quantity.saturating_sub(incoming_quantity);
                let consumed = visible_quantity - new_visible;

                self.quantity_policy.update_visible_quantity(new_visible);
                if !new_visible.is_zero() {
                    (consumed, Quantity(0))
                } else {
                    // Try replenishing the order
                    let replenished = self.quantity_policy.replenish();
                    (consumed, replenished)
                }
            }
        }
    }
}

impl Deref for LimitOrder {
    type Target = LimitOrderSpec;

    fn deref(&self) -> &Self::Target {
        &self.spec
    }
}
impl DerefMut for LimitOrder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.spec
    }
}

impl fmt::Display for LimitOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.quantity_policy {
            QuantityPolicy::Standard { quantity } => {
                write!(
                    f,
                    "Standard: id={} price={} quantity={} side={} post_only={} time_in_force={}",
                    self.id(),
                    self.price,
                    quantity,
                    self.side(),
                    self.post_only(),
                    self.time_in_force()
                )
            }
            QuantityPolicy::Iceberg {
                visible_quantity,
                hidden_quantity,
                replenish_quantity,
            } => {
                write!(
                    f,
                    "Iceberg: id={} price={} visible_quantity={} hidden_quantity={} replenish_quantity={} side={} post_only={} time_in_force={}",
                    self.id(),
                    self.price,
                    visible_quantity,
                    hidden_quantity,
                    replenish_quantity,
                    self.side(),
                    self.post_only(),
                    self.time_in_force()
                )
            }
        }
    }
}

/// Specification of a limit order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LimitOrderSpec {
    /// The price of the order
    price: Price,
    /// The quantity policy of the order
    quantity_policy: QuantityPolicy,
    /// The flags of the order
    flags: OrderFlags,
}

impl LimitOrderSpec {
    /// Create a new limit order specification
    pub fn new(price: Price, quantity_policy: QuantityPolicy, flags: OrderFlags) -> Self {
        Self {
            price,
            quantity_policy,
            flags,
        }
    }
    /// Get the price
    pub fn price(&self) -> Price {
        self.price
    }

    /// Update the price of the order
    pub(crate) fn update_price(&mut self, new_price: Price) {
        self.price = new_price;
    }

    /// Get the quantity policy
    pub fn quantity_policy(&self) -> QuantityPolicy {
        self.quantity_policy
    }

    /// Get the visible quantity
    pub fn visible_quantity(&self) -> Quantity {
        self.quantity_policy.visible_quantity()
    }

    /// Get the hidden quantity
    pub fn hidden_quantity(&self) -> Quantity {
        self.quantity_policy.hidden_quantity()
    }

    /// Get the replenish quantity
    pub fn replenish_quantity(&self) -> Quantity {
        self.quantity_policy.replenish_quantity()
    }

    /// Get the total quantity of the order
    pub fn total_quantity(&self) -> Quantity {
        self.quantity_policy.total_quantity()
    }

    /// Check if the order is filled
    pub fn is_filled(&self) -> bool {
        self.quantity_policy.is_filled()
    }

    /// Update the quantity policy of the order
    pub(crate) fn update_quantity_policy(&mut self, new_quantity_policy: QuantityPolicy) {
        self.quantity_policy = new_quantity_policy;
    }

    /// Get the flags of the order
    pub fn flags(&self) -> &OrderFlags {
        &self.flags
    }
}

impl Deref for LimitOrderSpec {
    type Target = OrderFlags;

    fn deref(&self) -> &Self::Target {
        &self.flags
    }
}
impl DerefMut for LimitOrderSpec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.flags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Quantity, QuantityPolicy, Side, TimeInForce, Timestamp, orders::OrderFlags};

    fn create_standard_order() -> LimitOrder {
        LimitOrder::new(
            OrderId(0),
            LimitOrderSpec::new(
                Price(90),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
            ),
        )
    }

    fn create_iceberg_order() -> LimitOrder {
        LimitOrder::new(
            OrderId(1),
            LimitOrderSpec::new(
                Price(100),
                QuantityPolicy::Iceberg {
                    visible_quantity: Quantity(20),
                    hidden_quantity: Quantity(40),
                    replenish_quantity: Quantity(20),
                },
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            ),
        )
    }

    #[test]
    fn test_id() {
        assert_eq!(create_standard_order().id(), OrderId(0));
        assert_eq!(create_iceberg_order().id(), OrderId(1));
    }

    #[test]
    fn test_price() {
        let mut order = create_standard_order();
        assert_eq!(order.price(), Price(90));

        order.update_price(Price(95));
        assert_eq!(order.price(), Price(95));

        assert_eq!(create_iceberg_order().price(), Price(100));
    }

    #[test]
    fn test_quantity_policy() {
        {
            let mut order = create_standard_order();
            assert_eq!(order.visible_quantity(), Quantity(10));
            assert_eq!(order.hidden_quantity(), Quantity(0));
            assert_eq!(order.replenish_quantity(), Quantity(0));
            assert_eq!(order.total_quantity(), Quantity(10));
            assert!(!order.is_filled());

            order.update_quantity_policy(QuantityPolicy::Iceberg {
                visible_quantity: Quantity(1),
                hidden_quantity: Quantity(10),
                replenish_quantity: Quantity(1),
            });

            assert_eq!(order.visible_quantity(), Quantity(1));
            assert_eq!(order.hidden_quantity(), Quantity(10));
            assert_eq!(order.replenish_quantity(), Quantity(1));
            assert_eq!(order.total_quantity(), Quantity(11));
            assert!(!order.is_filled());
        }
        {
            let mut order = create_iceberg_order();
            assert_eq!(order.visible_quantity(), Quantity(20));
            assert_eq!(order.hidden_quantity(), Quantity(40));
            assert_eq!(order.replenish_quantity(), Quantity(20));
            assert_eq!(order.total_quantity(), Quantity(60));
            assert!(!order.is_filled());

            order.update_quantity_policy(QuantityPolicy::Standard {
                quantity: Quantity(100),
            });
            assert_eq!(order.visible_quantity(), Quantity(100));
            assert_eq!(order.hidden_quantity(), Quantity(0));
            assert_eq!(order.replenish_quantity(), Quantity(0));
            assert_eq!(order.total_quantity(), Quantity(100));
            assert!(!order.is_filled());
        }
    }

    #[test]
    fn test_side() {
        assert_eq!(create_standard_order().side(), Side::Buy);
        assert_eq!(create_iceberg_order().side(), Side::Sell);
    }

    #[test]
    fn test_post_only() {
        assert!(create_standard_order().post_only());
        assert!(!create_iceberg_order().post_only());
    }

    #[test]
    fn test_time_in_force() {
        let mut order = create_standard_order();
        assert_eq!(order.time_in_force(), TimeInForce::Gtc);
        assert!(!order.is_immediate());
        assert!(!order.has_expiry());
        assert!(!order.is_expired(Timestamp(1771180000)));

        order.update_time_in_force(TimeInForce::Ioc);
        assert!(order.is_immediate());
        assert!(!order.has_expiry());
        assert!(!order.is_expired(Timestamp(1771180000)));

        order.update_time_in_force(TimeInForce::Fok);
        assert!(order.is_immediate());
        assert!(!order.has_expiry());
        assert!(!order.is_expired(Timestamp(1771180000)));

        order.update_time_in_force(TimeInForce::Gtd(Timestamp(1771180000 + 1000)));
        assert!(!order.is_immediate());
        assert!(order.has_expiry());
        assert!(!order.is_expired(Timestamp(1771180000)));
        assert!(order.is_expired(Timestamp(1771180000 + 1000)));
    }

    #[test]
    fn test_match_against() {
        {
            let mut order = create_standard_order();
            assert_eq!(order.visible_quantity(), Quantity(10));
            assert_eq!(order.hidden_quantity(), Quantity(0));
            assert_eq!(order.replenish_quantity(), Quantity(0));

            let (consumed, replenished) = order.match_against(Quantity(2));
            assert_eq!(consumed, Quantity(2));
            assert_eq!(replenished, Quantity(0));
            assert_eq!(order.visible_quantity(), Quantity(8));
            assert_eq!(order.hidden_quantity(), Quantity(0));
            assert_eq!(order.replenish_quantity(), Quantity(0));

            let (consumed, replenished) = order.match_against(Quantity(10));
            assert_eq!(consumed, Quantity(8));
            assert_eq!(replenished, Quantity(0));
            assert_eq!(order.visible_quantity(), Quantity(0));
            assert_eq!(order.hidden_quantity(), Quantity(0));
            assert_eq!(order.replenish_quantity(), Quantity(0));

            let (consumed, replenished) = order.match_against(Quantity(10));
            assert_eq!(consumed, Quantity(0));
            assert_eq!(replenished, Quantity(0));
            assert_eq!(order.visible_quantity(), Quantity(0));
            assert_eq!(order.hidden_quantity(), Quantity(0));
            assert_eq!(order.replenish_quantity(), Quantity(0));
        }
        {
            let mut order = create_iceberg_order();
            assert_eq!(order.visible_quantity(), Quantity(20));
            assert_eq!(order.hidden_quantity(), Quantity(40));
            assert_eq!(order.replenish_quantity(), Quantity(20));

            let (consumed, replenished) = order.match_against(Quantity(5));
            assert_eq!(consumed, Quantity(5));
            assert_eq!(replenished, Quantity(0));
            assert_eq!(order.visible_quantity(), Quantity(15));
            assert_eq!(order.hidden_quantity(), Quantity(40));
            assert_eq!(order.replenish_quantity(), Quantity(20));

            let (consumed, replenished) = order.match_against(Quantity(20));
            assert_eq!(consumed, Quantity(15));
            assert_eq!(replenished, Quantity(20));
            assert_eq!(order.visible_quantity(), Quantity(20));
            assert_eq!(order.hidden_quantity(), Quantity(20));
            assert_eq!(order.replenish_quantity(), Quantity(20));

            let (consumed, replenished) = order.match_against(Quantity(20));
            assert_eq!(consumed, Quantity(20));
            assert_eq!(replenished, Quantity(20));
            assert_eq!(order.visible_quantity(), Quantity(20));
            assert_eq!(order.hidden_quantity(), Quantity(0));
            assert_eq!(order.replenish_quantity(), Quantity(20));

            let (consumed, replenished) = order.match_against(Quantity(1));
            assert_eq!(consumed, Quantity(1));
            assert_eq!(replenished, Quantity(0));
            assert_eq!(order.visible_quantity(), Quantity(19));
            assert_eq!(order.hidden_quantity(), Quantity(0));
            assert_eq!(order.replenish_quantity(), Quantity(20));

            let (consumed, replenished) = order.match_against(Quantity(19));
            assert_eq!(consumed, Quantity(19));
            assert_eq!(replenished, Quantity(0));
            assert_eq!(order.visible_quantity(), Quantity(0));
            assert_eq!(order.hidden_quantity(), Quantity(0));
            assert_eq!(order.replenish_quantity(), Quantity(20));

            let (consumed, replenished) = order.match_against(Quantity(1));
            assert_eq!(consumed, Quantity(0));
            assert_eq!(replenished, Quantity(0));
            assert_eq!(order.visible_quantity(), Quantity(0));
            assert_eq!(order.hidden_quantity(), Quantity(0));
            assert_eq!(order.replenish_quantity(), Quantity(20));
        }
    }

    #[test]
    fn test_roundtrip_serialization() {
        for order in [create_standard_order(), create_iceberg_order()] {
            let serialized = serde_json::to_string(&order).unwrap();
            let deserialized: LimitOrder = serde_json::from_str(&serialized).unwrap();
            assert_eq!(order, deserialized);
        }
    }

    #[test]
    fn test_display() {
        assert_eq!(
            create_standard_order().to_string(),
            "Standard: id=0 price=90 quantity=10 side=BUY post_only=true time_in_force=GTC"
        );
        assert_eq!(
            create_iceberg_order().to_string(),
            "Iceberg: id=1 price=100 visible_quantity=20 hidden_quantity=40 replenish_quantity=20 side=SELL post_only=false time_in_force=GTC"
        );
    }
}

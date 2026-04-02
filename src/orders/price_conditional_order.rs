use super::{LimitOrder, MarketOrder};
use crate::{LevelId, Price, SequenceNumber, Timestamp};

use std::ops::{Deref, DerefMut};

/// Represents a resting price-conditional order
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestingPriceConditionalOrder {
    /// The time priority of the order
    time_priority: SequenceNumber,
    /// The ID of the level the order is resting at
    level_id: LevelId,
    /// The price-conditional order
    inner: PriceConditionalOrder,
}

impl RestingPriceConditionalOrder {
    /// Create a new resting price-conditional order
    pub fn new(
        time_priority: SequenceNumber,
        level_id: LevelId,
        inner: PriceConditionalOrder,
    ) -> Self {
        Self {
            time_priority,
            level_id,
            inner,
        }
    }

    /// Get the time priority of the order
    pub fn time_priority(&self) -> SequenceNumber {
        self.time_priority
    }

    /// Update the time priority of the order
    pub(crate) fn update_time_priority(&mut self, new_time_priority: SequenceNumber) {
        self.time_priority = new_time_priority;
    }

    /// Get the ID of the level the order is resting at
    pub fn level_id(&self) -> LevelId {
        self.level_id
    }

    /// Update the ID of the level the order is resting at
    pub(crate) fn update_level_id(&mut self, new_level_id: LevelId) {
        self.level_id = new_level_id;
    }

    /// Get the price-conditional order
    pub fn inner(&self) -> &PriceConditionalOrder {
        &self.inner
    }

    /// Convert the resting price-conditional order into a price-conditional order
    pub fn into_inner(self) -> PriceConditionalOrder {
        self.inner
    }
}

impl Deref for RestingPriceConditionalOrder {
    type Target = PriceConditionalOrder;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl DerefMut for RestingPriceConditionalOrder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Represents a price-conditional order
///
/// A price-conditional order remains inactive until a specified price condition is satisfied.
/// For example, the order may be activated when the market price is at or above (or at or below) a given trigger price.
///
/// Once the condition is met, the order is activated and a new order is submitted to the order book.
/// The resulting order is treated as a fresh submission with its own time priority.
///
/// The activated order can be either a market order or a limit order, allowing
/// this type to model a variety of common conditional orders, including:
///
/// - Stop-loss orders
/// - Take-profit orders
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriceConditionalOrder {
    /// The condition that must be met for the order to be activated
    price_condition: PriceCondition,
    /// The target order to execute when the condition is met
    target_order: TriggerOrder,
}

impl PriceConditionalOrder {
    /// Create a new price-conditional order
    pub fn new(price_condition: PriceCondition, target_order: TriggerOrder) -> Self {
        Self {
            price_condition,
            target_order,
        }
    }

    /// Get the condition that must be met for the order to be activated
    pub fn price_condition(&self) -> PriceCondition {
        self.price_condition
    }

    /// Update the condition that must be met for the order to be activated
    pub(crate) fn update_price_condition(&mut self, new_price_condition: PriceCondition) {
        self.price_condition = new_price_condition;
    }

    /// Get the target order to execute when the condition is met
    pub fn target_order(&self) -> &TriggerOrder {
        &self.target_order
    }

    /// Convert the price-conditional order into its target order to execute when the condition is met
    pub fn into_target_order(self) -> TriggerOrder {
        self.target_order
    }

    /// Update the target order to execute when the condition is met
    pub(crate) fn update_target_order(&mut self, new_target_order: TriggerOrder) {
        self.target_order = new_target_order;
    }

    /// Check if the target order is expired at a given timestamp
    pub fn is_expired(&self, timestamp: Timestamp) -> bool {
        self.target_order.is_expired(timestamp)
    }

    /// Check if the price-conditional order is ready to be activated at a given price
    pub fn is_ready(&self, price: Price) -> bool {
        self.price_condition.is_met(price)
    }
}

impl Deref for PriceConditionalOrder {
    type Target = PriceCondition;

    fn deref(&self) -> &Self::Target {
        &self.price_condition
    }
}
impl DerefMut for PriceConditionalOrder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.price_condition
    }
}

/// Represents the condition that must be met for a price-conditional order to be activated
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PriceCondition {
    /// The reference price that defines when the condition activates
    trigger_price: Price,
    /// The condition direction:
    /// - `AtOrAbove`: activates when market price >= `trigger_price`
    /// - `AtOrBelow`: activates when market price <= `trigger_price`
    direction: TriggerDirection,
}

impl PriceCondition {
    /// Create a new price condition
    pub fn new(trigger_price: Price, direction: TriggerDirection) -> Self {
        Self {
            trigger_price,
            direction,
        }
    }

    /// Get the reference price that defines when the condition activates
    pub fn trigger_price(&self) -> Price {
        self.trigger_price
    }

    /// Get the direction in which the price must move relative to `trigger_price`
    pub fn direction(&self) -> TriggerDirection {
        self.direction
    }

    /// Check if the price condition is met at a given price
    pub fn is_met(&self, price: Price) -> bool {
        match self.direction() {
            TriggerDirection::AtOrAbove => price >= self.trigger_price(),
            TriggerDirection::AtOrBelow => price <= self.trigger_price(),
        }
    }
}

/// Direction of trigger evaluation relative to the trigger price
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerDirection {
    /// Trigger when the observed price >= trigger_price
    AtOrAbove,
    /// Trigger when the observed price <= trigger_price
    AtOrBelow,
}

/// Represents the order to execute when the condition is met
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriggerOrder {
    /// Execute a market order
    Market(MarketOrder),
    /// Execute a limit order
    Limit(LimitOrder),
}

impl TriggerOrder {
    /// Check if the order is expired at a given timestamp
    pub fn is_expired(&self, timestamp: Timestamp) -> bool {
        match self {
            TriggerOrder::Market(_) => false,
            TriggerOrder::Limit(order) => order.is_expired(timestamp),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_is_expired() {
        let test_ts = 1771180000;

        struct Case {
            name: &'static str,
            order: PriceConditionalOrder,
            expected: bool,
        }

        let cases = [
            Case {
                name: "market order",
                order: PriceConditionalOrder::new(
                    PriceCondition::new(Price(100), TriggerDirection::AtOrAbove),
                    TriggerOrder::Market(MarketOrder::new(Quantity(100), Side::Buy, true)),
                ),
                expected: false,
            },
            Case {
                name: "limit order (GTC)",
                order: PriceConditionalOrder::new(
                    PriceCondition::new(Price(100), TriggerDirection::AtOrAbove),
                    TriggerOrder::Limit(LimitOrder::new(
                        Price(100),
                        QuantityPolicy::Standard {
                            quantity: Quantity(100),
                        },
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    )),
                ),
                expected: false,
            },
            Case {
                name: "limit order (unexpired GTD)",
                order: PriceConditionalOrder::new(
                    PriceCondition::new(Price(100), TriggerDirection::AtOrAbove),
                    TriggerOrder::Limit(LimitOrder::new(
                        Price(100),
                        QuantityPolicy::Standard {
                            quantity: Quantity(100),
                        },
                        OrderFlags::new(
                            Side::Buy,
                            false,
                            TimeInForce::Gtd(Timestamp(test_ts + 1000)),
                        ),
                    )),
                ),
                expected: false,
            },
            Case {
                name: "limit order (expired GTD)",
                order: PriceConditionalOrder::new(
                    PriceCondition::new(Price(100), TriggerDirection::AtOrAbove),
                    TriggerOrder::Limit(LimitOrder::new(
                        Price(100),
                        QuantityPolicy::Standard {
                            quantity: Quantity(100),
                        },
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtd(Timestamp(test_ts))),
                    )),
                ),
                expected: true,
            },
        ];

        for case in cases {
            assert_eq!(
                case.order.is_expired(Timestamp(test_ts)),
                case.expected,
                "case: {}",
                case.name
            );
        }
    }

    #[test]
    fn test_is_met() {
        struct Case {
            name: &'static str,
            price_condition: PriceCondition,
            market_price: Price,
            expected: bool,
        }

        let cases = [
            Case {
                name: "at or above trigger price",
                price_condition: PriceCondition::new(Price(100), TriggerDirection::AtOrAbove),
                market_price: Price(100),
                expected: true,
            },
            Case {
                name: "at or below trigger price",
                price_condition: PriceCondition::new(Price(100), TriggerDirection::AtOrBelow),
                market_price: Price(100),
                expected: true,
            },
            Case {
                name: "at or above trigger price (not ready)",
                price_condition: PriceCondition::new(Price(100), TriggerDirection::AtOrAbove),
                market_price: Price(99),
                expected: false,
            },
            Case {
                name: "at or below trigger price (not ready)",
                price_condition: PriceCondition::new(Price(100), TriggerDirection::AtOrBelow),
                market_price: Price(101),
                expected: false,
            },
        ];

        for case in cases {
            assert_eq!(
                case.price_condition.is_met(case.market_price),
                case.expected,
                "case: {}",
                case.name
            );
        }
    }
}

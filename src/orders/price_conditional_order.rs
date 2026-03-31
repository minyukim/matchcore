use super::{LimitOrder, MarketOrder};
use crate::{LevelId, Price, SequenceNumber};

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
    order: PriceConditionalOrder,
}

impl RestingPriceConditionalOrder {
    /// Create a new resting price-conditional order
    pub fn new(
        time_priority: SequenceNumber,
        level_id: LevelId,
        order: PriceConditionalOrder,
    ) -> Self {
        Self {
            time_priority,
            level_id,
            order,
        }
    }

    /// Get the time priority of the order
    pub fn time_priority(&self) -> SequenceNumber {
        self.time_priority
    }

    /// Get the ID of the level the order is resting at
    pub fn level_id(&self) -> LevelId {
        self.level_id
    }

    /// Get the price-conditional order
    pub fn order(&self) -> &PriceConditionalOrder {
        &self.order
    }

    /// Convert the resting price-conditional order into a price-conditional order
    pub fn into_order(self) -> PriceConditionalOrder {
        self.order
    }
}

impl Deref for RestingPriceConditionalOrder {
    type Target = PriceConditionalOrder;

    fn deref(&self) -> &Self::Target {
        &self.order
    }
}
impl DerefMut for RestingPriceConditionalOrder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.order
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
    /// The trigger price threshold
    trigger_price: Price,
    /// The direction in which the price must move relative to `trigger_price`
    direction: TriggerDirection,
    /// The target order to execute when the condition is met
    target_order: TriggerOrder,
}

impl PriceConditionalOrder {
    /// Create a new price-conditional order
    pub fn new(
        trigger_price: Price,
        direction: TriggerDirection,
        target_order: TriggerOrder,
    ) -> Self {
        Self {
            trigger_price,
            direction,
            target_order,
        }
    }

    /// Get the trigger price threshold
    pub fn trigger_price(&self) -> Price {
        self.trigger_price
    }

    /// Get the direction in which the price must move relative to `trigger_price`
    pub fn direction(&self) -> TriggerDirection {
        self.direction
    }

    /// Get the target order to execute when the condition is met
    pub fn target_order(&self) -> &TriggerOrder {
        &self.target_order
    }

    /// Convert the price-conditional order into its target order to execute when the condition is met
    pub fn into_target_order(self) -> TriggerOrder {
        self.target_order
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

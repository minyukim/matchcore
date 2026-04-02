use super::{
    CommandError,
    validation::{
        validate_limit_order_invariants, validate_pegged_order_invariants,
        validate_price_conditional_order_invariants,
    },
};
use crate::{orders::*, types::*};

use std::ops::{Deref, DerefMut};

/// Represents a command to amend an existing order
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmendCmd {
    /// The ID of the order to amend
    pub order_id: OrderId,
    /// The patch to apply to the order
    pub patch: AmendPatch,
}

/// Represents the patch to an existing order
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AmendPatch {
    /// The patch to a limit order
    Limit(LimitOrderPatch),
    /// The patch to a pegged order
    Pegged(PeggedOrderPatch),
    /// The patch to a price-conditional order
    PriceConditional(PriceConditionalOrderPatch),
}

/// Represents the patch to a limit order
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LimitOrderPatch {
    /// The new price of the order
    pub price: Option<Price>,
    /// The new quantity policy of the order
    pub quantity_policy: Option<QuantityPolicy>,
    /// The flags to update
    pub flags: OrderFlagsPatch,
}

impl LimitOrderPatch {
    /// Create a new empty limit order patch
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns this patch with the price set.
    pub fn with_price(mut self, v: Price) -> Self {
        self.price = Some(v);
        self
    }

    /// Returns this patch with the quantity policy set.
    pub fn with_quantity_policy(mut self, v: QuantityPolicy) -> Self {
        self.quantity_policy = Some(v);
        self
    }

    /// Returns this patch with the post-only set.
    pub fn with_post_only(mut self, v: bool) -> Self {
        self.flags.post_only = Some(v);
        self
    }

    /// Returns this patch with the time-in-force set.
    pub fn with_time_in_force(mut self, v: TimeInForce) -> Self {
        self.flags.time_in_force = Some(v);
        self
    }

    /// Check if the patch is empty
    pub fn is_empty(&self) -> bool {
        self.flags.is_empty() && self.price.is_none() && self.quantity_policy.is_none()
    }

    /// Checks if the patch has expired time in force at a given timestamp
    pub fn has_expired_time_in_force(&self, timestamp: Timestamp) -> bool {
        self.flags.has_expired_time_in_force(timestamp)
    }

    /// Apply the patch to the order if the patch does not conflict with the order
    pub(crate) fn apply(&self, order: &mut LimitOrder) -> Result<(), CommandError> {
        let new_price = self.price.unwrap_or(order.price());
        let new_quantity_policy = self.quantity_policy.unwrap_or(order.quantity_policy());
        let new_post_only = self.flags.post_only.unwrap_or(order.post_only());
        let new_time_in_force = self.flags.time_in_force.unwrap_or(order.time_in_force());

        validate_limit_order_invariants(
            new_price,
            new_quantity_policy,
            new_post_only,
            new_time_in_force,
        )?;

        if order.price() == new_price && new_time_in_force.is_immediate() {
            return Err(CommandError::SameLevelImmediateTif);
        }

        order.update_price(new_price);
        order.update_quantity_policy(new_quantity_policy);
        order.update_post_only(new_post_only);
        order.update_time_in_force(new_time_in_force);

        Ok(())
    }
}

impl Deref for LimitOrderPatch {
    type Target = OrderFlagsPatch;

    fn deref(&self) -> &Self::Target {
        &self.flags
    }
}
impl DerefMut for LimitOrderPatch {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.flags
    }
}

/// Represents the patch to a pegged order
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PeggedOrderPatch {
    /// The new peg reference type
    pub peg_reference: Option<PegReference>,
    /// The new quantity of the order
    pub quantity: Option<Quantity>,
    /// The flags to update
    pub flags: OrderFlagsPatch,
}

impl PeggedOrderPatch {
    /// Create a new empty pegged order patch
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns this patch with the peg reference set.
    pub fn with_peg_reference(mut self, v: PegReference) -> Self {
        self.peg_reference = Some(v);
        self
    }

    /// Returns this patch with the quantity set.
    pub fn with_quantity(mut self, v: Quantity) -> Self {
        self.quantity = Some(v);
        self
    }

    /// Returns this patch with the post-only set.
    pub fn with_post_only(mut self, v: bool) -> Self {
        self.flags.post_only = Some(v);
        self
    }

    /// Returns this patch with the time-in-force set.
    pub fn with_time_in_force(mut self, v: TimeInForce) -> Self {
        self.flags.time_in_force = Some(v);
        self
    }

    /// Check if the patch is empty
    pub fn is_empty(&self) -> bool {
        self.flags.is_empty() && self.peg_reference.is_none() && self.quantity.is_none()
    }

    /// Checks if the patch has expired time in force at a given timestamp
    pub fn has_expired_time_in_force(&self, timestamp: Timestamp) -> bool {
        self.flags.has_expired_time_in_force(timestamp)
    }

    /// Apply the patch to the order if the patch does not conflict with the order
    pub(crate) fn apply(&self, order: &mut PeggedOrder) -> Result<(), CommandError> {
        let new_peg_reference = self.peg_reference.unwrap_or(order.peg_reference());
        let new_quantity = self.quantity.unwrap_or(order.quantity());
        let new_post_only = self.flags.post_only.unwrap_or(order.post_only());
        let new_time_in_force = self.flags.time_in_force.unwrap_or(order.time_in_force());

        validate_pegged_order_invariants(
            new_peg_reference,
            new_quantity,
            new_post_only,
            new_time_in_force,
        )?;

        if order.peg_reference() == new_peg_reference && new_time_in_force.is_immediate() {
            return Err(CommandError::SameLevelImmediateTif);
        }

        order.update_peg_reference(new_peg_reference);
        order.update_quantity(new_quantity);
        order.update_post_only(new_post_only);
        order.update_time_in_force(new_time_in_force);

        Ok(())
    }
}

impl Deref for PeggedOrderPatch {
    type Target = OrderFlagsPatch;

    fn deref(&self) -> &Self::Target {
        &self.flags
    }
}
impl DerefMut for PeggedOrderPatch {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.flags
    }
}

/// Represents the patch to the flags of an order
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OrderFlagsPatch {
    /// The new post-only flag
    pub post_only: Option<bool>,
    /// The new time in force of the order
    pub time_in_force: Option<TimeInForce>,
}

impl OrderFlagsPatch {
    /// Check if the patch is empty
    pub fn is_empty(&self) -> bool {
        self.post_only.is_none() && self.time_in_force.is_none()
    }

    /// Checks if the patch has expired time in force at a given timestamp
    pub fn has_expired_time_in_force(&self, timestamp: Timestamp) -> bool {
        self.time_in_force
            .is_some_and(|time_in_force| time_in_force.is_expired(timestamp))
    }
}

/// Represents the patch to a price-conditional order
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PriceConditionalOrderPatch {
    /// The new trigger price threshold of the order
    pub trigger_price: Option<Price>,
    /// The new direction in which the price must move relative to `trigger_price`
    pub direction: Option<TriggerDirection>,
    /// The new target order to execute when the condition is met
    pub target_order: Option<TriggerOrder>,
}

impl PriceConditionalOrderPatch {
    /// Create a new empty price-conditional order patch
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns this patch with the trigger price threshold set.
    pub fn with_trigger_price(mut self, v: Price) -> Self {
        self.trigger_price = Some(v);
        self
    }

    /// Returns this patch with the direction set.
    pub fn with_direction(mut self, v: TriggerDirection) -> Self {
        self.direction = Some(v);
        self
    }

    /// Returns this patch with the target order set.
    pub fn with_target_order(mut self, v: TriggerOrder) -> Self {
        self.target_order = Some(v);
        self
    }

    /// Check if the patch is empty
    pub fn is_empty(&self) -> bool {
        self.trigger_price.is_none() && self.direction.is_none() && self.target_order.is_none()
    }

    /// Checks if the patch has expired time in force at a given timestamp
    pub fn has_expired_time_in_force(&self, timestamp: Timestamp) -> bool {
        self.target_order
            .as_ref()
            .is_some_and(|target_order| target_order.is_expired(timestamp))
    }

    /// Apply the patch to the order if the patch does not conflict with the order
    #[allow(dead_code)]
    pub(crate) fn apply(&self, order: &mut PriceConditionalOrder) -> Result<(), CommandError> {
        let new_trigger_price = self.trigger_price.unwrap_or(order.trigger_price());
        let new_direction = self.direction.unwrap_or(order.direction());
        let new_target_order = self.target_order.as_ref().unwrap_or(order.target_order());

        validate_price_conditional_order_invariants(new_trigger_price, new_target_order)?;

        order.update_target_order(new_target_order.clone());
        order.update_trigger_price(new_trigger_price);
        order.update_direction(new_direction);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        LimitOrder, MarketOrder, OrderFlags, PeggedOrder, PriceConditionalOrder, Side,
        TriggerDirection, TriggerOrder,
    };

    #[test]
    fn test_is_empty_limit_order_patch() {
        let patch = LimitOrderPatch::new();
        assert!(patch.is_empty());
        let patch = LimitOrderPatch::new().with_price(Price(100));
        assert!(!patch.is_empty());
        let patch = LimitOrderPatch::new().with_quantity_policy(QuantityPolicy::Standard {
            quantity: Quantity(10),
        });
        assert!(!patch.is_empty());
        let patch = LimitOrderPatch::new().with_post_only(true);
        assert!(!patch.is_empty());
        let patch = LimitOrderPatch::new().with_time_in_force(TimeInForce::Gtc);
        assert!(!patch.is_empty());
        let patch = LimitOrderPatch::new()
            .with_post_only(true)
            .with_time_in_force(TimeInForce::Gtc);
        assert!(!patch.is_empty());
    }

    #[test]
    fn test_apply_limit_order_patch() {
        struct Case {
            name: &'static str,
            order: LimitOrder,
            patch: LimitOrderPatch,
            expected: Result<(), CommandError>,
        }

        let cases = [
            Case {
                name: "no-op patch (same price and quantity policy)",
                order: LimitOrder::new(
                    Price(100),
                    QuantityPolicy::Standard {
                        quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                ),
                patch: LimitOrderPatch::new()
                    .with_price(Price(100))
                    .with_quantity_policy(QuantityPolicy::Standard {
                        quantity: Quantity(10),
                    }),
                expected: Ok(()),
            },
            Case {
                name: "update price only",
                order: LimitOrder::new(
                    Price(100),
                    QuantityPolicy::Standard {
                        quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                ),
                patch: LimitOrderPatch::new().with_price(Price(200)),
                expected: Ok(()),
            },
            Case {
                name: "update quantity policy only",
                order: LimitOrder::new(
                    Price(100),
                    QuantityPolicy::Standard {
                        quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                ),
                patch: LimitOrderPatch::new().with_quantity_policy(QuantityPolicy::Standard {
                    quantity: Quantity(20),
                }),
                expected: Ok(()),
            },
            Case {
                name: "update post_only only",
                order: LimitOrder::new(
                    Price(100),
                    QuantityPolicy::Standard {
                        quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                ),
                patch: LimitOrderPatch::new().with_post_only(true),
                expected: Ok(()),
            },
            Case {
                name: "update time_in_force only",
                order: LimitOrder::new(
                    Price(100),
                    QuantityPolicy::Standard {
                        quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                ),
                patch: LimitOrderPatch::new().with_time_in_force(TimeInForce::Gtd(Timestamp(100))),
                expected: Ok(()),
            },
            Case {
                name: "invalid: post-only with immediate TIF",
                order: LimitOrder::new(
                    Price(100),
                    QuantityPolicy::Standard {
                        quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                ),
                patch: LimitOrderPatch::new()
                    .with_post_only(true)
                    .with_time_in_force(TimeInForce::Ioc),
                expected: Err(CommandError::PostOnlyImmediateTif),
            },
            Case {
                name: "invalid: zero price",
                order: LimitOrder::new(
                    Price(100),
                    QuantityPolicy::Standard {
                        quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                ),
                patch: LimitOrderPatch::new().with_price(Price(0)),
                expected: Err(CommandError::ZeroPrice),
            },
            Case {
                name: "invalid: zero quantity",
                order: LimitOrder::new(
                    Price(100),
                    QuantityPolicy::Standard {
                        quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                ),
                patch: LimitOrderPatch::new()
                    .with_price(Price(100))
                    .with_quantity_policy(QuantityPolicy::Standard {
                        quantity: Quantity(0),
                    }),
                expected: Err(CommandError::ZeroQuantity),
            },
            Case {
                name: "invalid: iceberg with immediate TIF",
                order: LimitOrder::new(
                    Price(100),
                    QuantityPolicy::Standard {
                        quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                ),
                patch: LimitOrderPatch::new()
                    .with_quantity_policy(QuantityPolicy::Iceberg {
                        visible_quantity: Quantity(10),
                        hidden_quantity: Quantity(10),
                        replenish_quantity: Quantity(10),
                    })
                    .with_time_in_force(TimeInForce::Ioc),
                expected: Err(CommandError::IcebergImmediateTif),
            },
            Case {
                name: "valid patch + valid order → invalid: order is post_only, patch sets immediate TIF",
                order: LimitOrder::new(
                    Price(100),
                    QuantityPolicy::Standard {
                        quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
                patch: LimitOrderPatch::new().with_time_in_force(TimeInForce::Ioc),
                expected: Err(CommandError::PostOnlyImmediateTif),
            },
            Case {
                name: "valid patch + valid order → invalid: order is immediate TIF, patch sets post_only",
                order: LimitOrder::new(
                    Price(100),
                    QuantityPolicy::Standard {
                        quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Buy, false, TimeInForce::Ioc),
                ),
                patch: LimitOrderPatch::new().with_post_only(true),
                expected: Err(CommandError::PostOnlyImmediateTif),
            },
            Case {
                name: "valid patch + valid order → invalid: order is iceberg, patch sets immediate TIF",
                order: LimitOrder::new(
                    Price(100),
                    QuantityPolicy::Iceberg {
                        visible_quantity: Quantity(10),
                        hidden_quantity: Quantity(10),
                        replenish_quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                ),
                patch: LimitOrderPatch::new().with_time_in_force(TimeInForce::Ioc),
                expected: Err(CommandError::IcebergImmediateTif),
            },
            Case {
                name: "valid patch + valid order → invalid: order stays at the same level, patch sets immediate TIF",
                order: LimitOrder::new(
                    Price(100),
                    QuantityPolicy::Standard {
                        quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                ),
                patch: LimitOrderPatch::new().with_time_in_force(TimeInForce::Ioc),
                expected: Err(CommandError::SameLevelImmediateTif),
            },
        ];

        for case in cases {
            let mut order = case.order.clone();
            let result = case.patch.apply(&mut order);

            match (&case.expected, &result) {
                (Ok(()), Ok(())) => {
                    // Verify order was updated as expected
                    let expected_price = case.patch.price.unwrap_or(case.order.price());
                    let expected_quantity_policy = case
                        .patch
                        .quantity_policy
                        .unwrap_or(case.order.quantity_policy());
                    let expected_post_only =
                        case.patch.flags.post_only.unwrap_or(case.order.post_only());
                    let expected_time_in_force = case
                        .patch
                        .flags
                        .time_in_force
                        .unwrap_or(case.order.time_in_force());

                    assert_eq!(order.price(), expected_price, "case: {}", case.name);
                    assert_eq!(
                        order.quantity_policy(),
                        expected_quantity_policy,
                        "case: {}",
                        case.name
                    );
                    assert_eq!(order.post_only(), expected_post_only, "case: {}", case.name);
                    assert_eq!(
                        order.time_in_force(),
                        expected_time_in_force,
                        "case: {}",
                        case.name
                    );
                }
                (Err(expected_err), Err(actual_err)) => {
                    assert_eq!(actual_err, expected_err, "case: {}", case.name);
                    // Order should be unchanged on error
                    assert_eq!(order.price(), case.order.price(), "case: {}", case.name);
                    assert_eq!(
                        order.quantity_policy(),
                        case.order.quantity_policy(),
                        "case: {}",
                        case.name
                    );
                }
                (expected, actual) => {
                    panic!(
                        "case: {}: expected {:?}, got {:?}",
                        case.name, expected, actual
                    );
                }
            }
        }
    }

    #[test]
    fn test_is_empty_pegged_order_patch() {
        let patch = PeggedOrderPatch::new();
        assert!(patch.is_empty());
        let patch = PeggedOrderPatch::new().with_peg_reference(PegReference::Market);
        assert!(!patch.is_empty());
        let patch = PeggedOrderPatch::new().with_quantity(Quantity(10));
        assert!(!patch.is_empty());
        let patch = PeggedOrderPatch::new().with_post_only(true);
        assert!(!patch.is_empty());
        let patch = PeggedOrderPatch::new().with_time_in_force(TimeInForce::Gtc);
        assert!(!patch.is_empty());
        let patch = PeggedOrderPatch::new()
            .with_post_only(true)
            .with_time_in_force(TimeInForce::Gtc);
        assert!(!patch.is_empty());
    }

    #[test]
    fn test_apply_pegged_order_patch() {
        struct Case {
            name: &'static str,
            order: PeggedOrder,
            patch: PeggedOrderPatch,
            expected: Result<(), CommandError>,
        }

        let cases = [
            Case {
                name: "no-op patch (same peg_reference and quantity)",
                order: PeggedOrder::new(
                    PegReference::Market,
                    Quantity(10),
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                ),
                patch: PeggedOrderPatch::new()
                    .with_peg_reference(PegReference::Market)
                    .with_quantity(Quantity(10)),
                expected: Ok(()),
            },
            Case {
                name: "update peg_reference only",
                order: PeggedOrder::new(
                    PegReference::Primary,
                    Quantity(10),
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                ),
                patch: PeggedOrderPatch::new().with_peg_reference(PegReference::Market),
                expected: Ok(()),
            },
            Case {
                name: "update quantity only",
                order: PeggedOrder::new(
                    PegReference::Market,
                    Quantity(10),
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                ),
                patch: PeggedOrderPatch::new().with_quantity(Quantity(20)),
                expected: Ok(()),
            },
            Case {
                name: "update post_only only",
                order: PeggedOrder::new(
                    PegReference::Primary,
                    Quantity(10),
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                ),
                patch: PeggedOrderPatch::new().with_post_only(true),
                expected: Ok(()),
            },
            Case {
                name: "update time_in_force only",
                order: PeggedOrder::new(
                    PegReference::Market,
                    Quantity(10),
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                ),
                patch: PeggedOrderPatch::new().with_time_in_force(TimeInForce::Gtd(Timestamp(100))),
                expected: Ok(()),
            },
            Case {
                name: "invalid: post-only with immediate TIF",
                order: PeggedOrder::new(
                    PegReference::Market,
                    Quantity(10),
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                ),
                patch: PeggedOrderPatch::new()
                    .with_post_only(true)
                    .with_time_in_force(TimeInForce::Ioc),
                expected: Err(CommandError::PostOnlyImmediateTif),
            },
            Case {
                name: "invalid: zero quantity",
                order: PeggedOrder::new(
                    PegReference::Market,
                    Quantity(10),
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                ),
                patch: PeggedOrderPatch::new().with_quantity(Quantity(0)),
                expected: Err(CommandError::ZeroQuantity),
            },
            Case {
                name: "invalid: peg reference Primary + immediate TIF",
                order: PeggedOrder::new(
                    PegReference::Market,
                    Quantity(10),
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                ),
                patch: PeggedOrderPatch::new()
                    .with_peg_reference(PegReference::Primary)
                    .with_time_in_force(TimeInForce::Ioc),
                expected: Err(CommandError::PeggedNonTakerImmediateTif),
            },
            Case {
                name: "invalid: peg reference Market + post-only",
                order: PeggedOrder::new(
                    PegReference::Primary,
                    Quantity(10),
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
                patch: PeggedOrderPatch::new()
                    .with_peg_reference(PegReference::Market)
                    .with_post_only(true),
                expected: Err(CommandError::PeggedAlwaysTakerPostOnly),
            },
            Case {
                name: "valid patch + valid order → invalid: order is post_only, patch sets immediate TIF",
                order: PeggedOrder::new(
                    PegReference::Primary,
                    Quantity(10),
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
                patch: PeggedOrderPatch::new().with_time_in_force(TimeInForce::Ioc),
                expected: Err(CommandError::PostOnlyImmediateTif),
            },
            Case {
                name: "valid patch + valid order → invalid: order is Primary, patch sets immediate TIF",
                order: PeggedOrder::new(
                    PegReference::Primary,
                    Quantity(10),
                    OrderFlags::new(Side::Buy, false, TimeInForce::Ioc),
                ),
                patch: PeggedOrderPatch::new().with_time_in_force(TimeInForce::Ioc),
                expected: Err(CommandError::PeggedNonTakerImmediateTif),
            },
            Case {
                name: "valid patch + valid order → invalid: order is post-only, patch sets peg reference to Market",
                order: PeggedOrder::new(
                    PegReference::Primary,
                    Quantity(10),
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
                patch: PeggedOrderPatch::new().with_peg_reference(PegReference::Market),
                expected: Err(CommandError::PeggedAlwaysTakerPostOnly),
            },
            Case {
                name: "valid patch + valid order → invalid: order stays at the same level, patch sets immediate TIF",
                order: PeggedOrder::new(
                    PegReference::Market,
                    Quantity(10),
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                ),
                patch: PeggedOrderPatch::new().with_time_in_force(TimeInForce::Ioc),
                expected: Err(CommandError::SameLevelImmediateTif),
            },
        ];

        for case in cases {
            let mut order = case.order.clone();
            let result = case.patch.apply(&mut order);

            match (&case.expected, &result) {
                (Ok(()), Ok(())) => {
                    let expected_peg_reference = case
                        .patch
                        .peg_reference
                        .unwrap_or(case.order.peg_reference());
                    let expected_quantity = case.patch.quantity.unwrap_or(case.order.quantity());
                    let expected_post_only =
                        case.patch.flags.post_only.unwrap_or(case.order.post_only());
                    let expected_time_in_force = case
                        .patch
                        .flags
                        .time_in_force
                        .unwrap_or(case.order.time_in_force());

                    assert_eq!(
                        order.peg_reference(),
                        expected_peg_reference,
                        "case: {}",
                        case.name
                    );
                    assert_eq!(order.quantity(), expected_quantity, "case: {}", case.name);
                    assert_eq!(order.post_only(), expected_post_only, "case: {}", case.name);
                    assert_eq!(
                        order.time_in_force(),
                        expected_time_in_force,
                        "case: {}",
                        case.name
                    );
                }
                (Err(expected_err), Err(actual_err)) => {
                    assert_eq!(actual_err, expected_err, "case: {}", case.name);
                    assert_eq!(
                        order.peg_reference(),
                        case.order.peg_reference(),
                        "case: {}",
                        case.name
                    );
                    assert_eq!(
                        order.quantity(),
                        case.order.quantity(),
                        "case: {}",
                        case.name
                    );
                }
                (expected, actual) => {
                    panic!(
                        "case: {}: expected {:?}, got {:?}",
                        case.name, expected, actual
                    );
                }
            }
        }
    }

    fn sample_price_conditional_order() -> PriceConditionalOrder {
        PriceConditionalOrder::new(
            PriceCondition::new(Price(100), TriggerDirection::AtOrAbove),
            TriggerOrder::Limit(LimitOrder::new(
                Price(50),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            )),
        )
    }

    #[test]
    fn test_is_empty_price_conditional_order_patch() {
        let patch = PriceConditionalOrderPatch::new();
        assert!(patch.is_empty());
        let patch = PriceConditionalOrderPatch::new().with_trigger_price(Price(100));
        assert!(!patch.is_empty());
        let patch = PriceConditionalOrderPatch::new().with_direction(TriggerDirection::AtOrBelow);
        assert!(!patch.is_empty());
        let patch = PriceConditionalOrderPatch::new().with_target_order(TriggerOrder::Market(
            MarketOrder::new(Quantity(10), Side::Buy, true),
        ));
        assert!(!patch.is_empty());
    }

    #[test]
    fn test_has_expired_time_in_force_price_conditional_order_patch() {
        let ts = Timestamp(1_000_000);

        assert!(
            !PriceConditionalOrderPatch::new().has_expired_time_in_force(ts),
            "empty patch"
        );

        let patch = PriceConditionalOrderPatch::new().with_target_order(TriggerOrder::Limit(
            LimitOrder::new(
                Price(50),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        ));
        assert!(!patch.has_expired_time_in_force(ts));

        let patch = PriceConditionalOrderPatch::new().with_target_order(TriggerOrder::Limit(
            LimitOrder::new(
                Price(50),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtd(ts)),
            ),
        ));
        assert!(patch.has_expired_time_in_force(ts));

        let patch = PriceConditionalOrderPatch::new().with_target_order(TriggerOrder::Limit(
            LimitOrder::new(
                Price(50),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtd(Timestamp(ts.0 + 1))),
            ),
        ));
        assert!(!patch.has_expired_time_in_force(ts));
    }

    #[test]
    fn test_apply_price_conditional_order_patch() {
        struct Case {
            name: &'static str,
            order: PriceConditionalOrder,
            patch: PriceConditionalOrderPatch,
            expected: Result<(), CommandError>,
        }

        let cases = [
            Case {
                name: "no-op patch (explicit same trigger, direction, target)",
                order: sample_price_conditional_order(),
                patch: PriceConditionalOrderPatch::new()
                    .with_trigger_price(Price(100))
                    .with_direction(TriggerDirection::AtOrAbove)
                    .with_target_order(TriggerOrder::Limit(LimitOrder::new(
                        Price(50),
                        QuantityPolicy::Standard {
                            quantity: Quantity(10),
                        },
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    ))),
                expected: Ok(()),
            },
            Case {
                name: "update trigger_price only",
                order: sample_price_conditional_order(),
                patch: PriceConditionalOrderPatch::new().with_trigger_price(Price(200)),
                expected: Ok(()),
            },
            Case {
                name: "update direction only",
                order: sample_price_conditional_order(),
                patch: PriceConditionalOrderPatch::new()
                    .with_direction(TriggerDirection::AtOrBelow),
                expected: Ok(()),
            },
            Case {
                name: "update target_order only",
                order: sample_price_conditional_order(),
                patch: PriceConditionalOrderPatch::new().with_target_order(TriggerOrder::Limit(
                    LimitOrder::new(
                        Price(60),
                        QuantityPolicy::Standard {
                            quantity: Quantity(20),
                        },
                        OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
                    ),
                )),
                expected: Ok(()),
            },
            Case {
                name: "invalid: zero trigger price from patch",
                order: sample_price_conditional_order(),
                patch: PriceConditionalOrderPatch::new().with_trigger_price(Price(0)),
                expected: Err(CommandError::ZeroTriggerPrice),
            },
            Case {
                name: "invalid: zero-quantity market target from patch",
                order: sample_price_conditional_order(),
                patch: PriceConditionalOrderPatch::new().with_target_order(TriggerOrder::Market(
                    MarketOrder::new(Quantity(0), Side::Buy, false),
                )),
                expected: Err(CommandError::ZeroQuantity),
            },
            Case {
                name: "invalid: zero-price limit target from patch",
                order: sample_price_conditional_order(),
                patch: PriceConditionalOrderPatch::new().with_target_order(TriggerOrder::Limit(
                    LimitOrder::new(
                        Price(0),
                        QuantityPolicy::Standard {
                            quantity: Quantity(10),
                        },
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    ),
                )),
                expected: Err(CommandError::ZeroPrice),
            },
        ];

        for case in cases {
            let mut order = case.order.clone();
            let result = case.patch.apply(&mut order);

            match (&case.expected, &result) {
                (Ok(()), Ok(())) => {
                    let expected_trigger_price = case
                        .patch
                        .trigger_price
                        .unwrap_or(case.order.trigger_price());
                    let expected_direction = case.patch.direction.unwrap_or(case.order.direction());
                    let expected_target_order = case
                        .patch
                        .target_order
                        .as_ref()
                        .unwrap_or(case.order.target_order())
                        .clone();

                    assert_eq!(
                        order.trigger_price(),
                        expected_trigger_price,
                        "case: {}",
                        case.name
                    );
                    assert_eq!(order.direction(), expected_direction, "case: {}", case.name);
                    assert_eq!(
                        order.target_order(),
                        &expected_target_order,
                        "case: {}",
                        case.name
                    );
                }
                (Err(expected_err), Err(actual_err)) => {
                    assert_eq!(actual_err, expected_err, "case: {}", case.name);
                    assert_eq!(
                        order.trigger_price(),
                        case.order.trigger_price(),
                        "case: {}",
                        case.name
                    );
                    assert_eq!(
                        order.direction(),
                        case.order.direction(),
                        "case: {}",
                        case.name
                    );
                    assert_eq!(
                        order.target_order(),
                        case.order.target_order(),
                        "case: {}",
                        case.name
                    );
                }
                (expected, actual) => {
                    panic!(
                        "case: {}: expected {:?}, got {:?}",
                        case.name, expected, actual
                    );
                }
            }
        }
    }

    #[test]
    fn test_has_expired_time_in_force() {
        struct Case {
            name: &'static str,
            patch: OrderFlagsPatch,
            timestamp: Timestamp,
            expected: bool,
        }

        let cases = [
            Case {
                name: "empty patch does not have expired time in force",
                patch: OrderFlagsPatch::default(),
                timestamp: Timestamp(1000),
                expected: false,
            },
            Case {
                name: "GTC patch does not have expired time in force",
                patch: OrderFlagsPatch {
                    post_only: None,
                    time_in_force: Some(TimeInForce::Gtc),
                },
                timestamp: Timestamp(1000),
                expected: false,
            },
            Case {
                name: "IOC patch does not have expired time in force",
                patch: OrderFlagsPatch {
                    post_only: None,
                    time_in_force: Some(TimeInForce::Ioc),
                },
                timestamp: Timestamp(1000),
                expected: false,
            },
            Case {
                name: "FOK patch does not have expired time in force",
                patch: OrderFlagsPatch {
                    post_only: None,
                    time_in_force: Some(TimeInForce::Fok),
                },
                timestamp: Timestamp(1000),
                expected: false,
            },
            Case {
                name: "GTD patch does not have expired time in force",
                patch: OrderFlagsPatch {
                    post_only: None,
                    time_in_force: Some(TimeInForce::Gtd(Timestamp(1000))),
                },
                timestamp: Timestamp(999),
                expected: false,
            },
            Case {
                name: "GTD order has expired time in force",
                patch: OrderFlagsPatch {
                    post_only: None,
                    time_in_force: Some(TimeInForce::Gtd(Timestamp(1000))),
                },
                timestamp: Timestamp(1000),
                expected: true,
            },
        ];

        for case in cases {
            let result = case.patch.has_expired_time_in_force(case.timestamp);
            assert_eq!(result, case.expected, "case: {}", case.name);
        }
    }
}

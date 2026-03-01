use crate::{
    LimitOrder, PegReference, PeggedOrder, QuantityPolicy, TimeInForce,
    command::{
        CommandError,
        validation::{validate_limit_order_invariants, validate_pegged_order_invariants},
    },
};

use serde::{Deserialize, Serialize};

/// Represents a command to amend an existing order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AmendCmd {
    /// The ID of the order to amend
    pub order_id: u64,
    /// The patch to apply to the order
    pub patch: AmendPatch,
}

/// Represents the patch to an existing order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmendPatch {
    /// The patch to a limit order
    Limit(LimitOrderPatch),
    /// The patch to a pegged order
    Pegged(PeggedOrderPatch),
}

/// Represents the patch to a limit order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LimitOrderPatch {
    /// The new price of the order
    pub price: Option<u64>,
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
    pub fn with_price(mut self, v: u64) -> Self {
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
    pub fn has_expired_time_in_force(&self, timestamp: u64) -> bool {
        self.flags.has_expired_time_in_force(timestamp)
    }

    /// Apply the patch to the order if the patch does not conflict with the order
    #[allow(unused)]
    pub(crate) fn apply(&self, order: &mut LimitOrder) -> Result<(), CommandError> {
        if self.is_empty() {
            return Err(CommandError::EmptyPatch);
        }

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

        order.update_price(new_price);
        order.update_quantity_policy(new_quantity_policy);
        order.update_post_only(new_post_only);
        order.update_time_in_force(new_time_in_force);

        Ok(())
    }
}

/// Represents the patch to a pegged order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PeggedOrderPatch {
    /// The new peg reference type
    pub peg_reference: Option<PegReference>,
    /// The new quantity of the order
    pub quantity: Option<u64>,
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
    pub fn with_quantity(mut self, v: u64) -> Self {
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
    pub fn has_expired_time_in_force(&self, timestamp: u64) -> bool {
        self.flags.has_expired_time_in_force(timestamp)
    }

    /// Apply the patch to the order if the patch does not conflict with the order
    #[allow(unused)]
    pub(crate) fn apply(&self, order: &mut PeggedOrder) -> Result<(), CommandError> {
        if self.is_empty() {
            return Err(CommandError::EmptyPatch);
        }

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

        order.update_peg_reference(new_peg_reference);
        order.update_quantity(new_quantity);
        order.update_post_only(new_post_only);
        order.update_time_in_force(new_time_in_force);

        Ok(())
    }
}

/// Represents the patch to the flags of an order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
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
    pub fn has_expired_time_in_force(&self, timestamp: u64) -> bool {
        self.time_in_force
            .is_some_and(|time_in_force| time_in_force.is_expired(timestamp))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LimitOrderSpec, OrderFlags, PeggedOrderSpec, Side};

    #[test]
    fn test_apply_limit_patch() {
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
                    1,
                    LimitOrderSpec::new(
                        100,
                        QuantityPolicy::Standard { quantity: 10 },
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    ),
                ),
                patch: LimitOrderPatch::new()
                    .with_price(100)
                    .with_quantity_policy(QuantityPolicy::Standard { quantity: 10 }),
                expected: Ok(()),
            },
            Case {
                name: "update price only",
                order: LimitOrder::new(
                    1,
                    LimitOrderSpec::new(
                        100,
                        QuantityPolicy::Standard { quantity: 10 },
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    ),
                ),
                patch: LimitOrderPatch::new().with_price(200),
                expected: Ok(()),
            },
            Case {
                name: "update quantity policy only",
                order: LimitOrder::new(
                    1,
                    LimitOrderSpec::new(
                        100,
                        QuantityPolicy::Standard { quantity: 10 },
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    ),
                ),
                patch: LimitOrderPatch::new()
                    .with_quantity_policy(QuantityPolicy::Standard { quantity: 20 }),
                expected: Ok(()),
            },
            Case {
                name: "update post_only only",
                order: LimitOrder::new(
                    1,
                    LimitOrderSpec::new(
                        100,
                        QuantityPolicy::Standard { quantity: 10 },
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    ),
                ),
                patch: LimitOrderPatch::new().with_post_only(true),
                expected: Ok(()),
            },
            Case {
                name: "update time_in_force only",
                order: LimitOrder::new(
                    1,
                    LimitOrderSpec::new(
                        100,
                        QuantityPolicy::Standard { quantity: 10 },
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    ),
                ),
                patch: LimitOrderPatch::new().with_time_in_force(TimeInForce::Ioc),
                expected: Ok(()),
            },
            Case {
                name: "invalid: empty patch",
                order: LimitOrder::new(
                    1,
                    LimitOrderSpec::new(
                        100,
                        QuantityPolicy::Standard { quantity: 10 },
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    ),
                ),
                patch: LimitOrderPatch::new(),
                expected: Err(CommandError::EmptyPatch),
            },
            Case {
                name: "invalid: post-only with immediate TIF",
                order: LimitOrder::new(
                    1,
                    LimitOrderSpec::new(
                        100,
                        QuantityPolicy::Standard { quantity: 10 },
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    ),
                ),
                patch: LimitOrderPatch::new()
                    .with_post_only(true)
                    .with_time_in_force(TimeInForce::Ioc),
                expected: Err(CommandError::PostOnlyImmediateTif),
            },
            Case {
                name: "invalid: zero price",
                order: LimitOrder::new(
                    1,
                    LimitOrderSpec::new(
                        100,
                        QuantityPolicy::Standard { quantity: 10 },
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    ),
                ),
                patch: LimitOrderPatch::new().with_price(0),
                expected: Err(CommandError::ZeroPrice),
            },
            Case {
                name: "invalid: zero quantity",
                order: LimitOrder::new(
                    1,
                    LimitOrderSpec::new(
                        100,
                        QuantityPolicy::Standard { quantity: 10 },
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    ),
                ),
                patch: LimitOrderPatch::new()
                    .with_price(100)
                    .with_quantity_policy(QuantityPolicy::Standard { quantity: 0 }),
                expected: Err(CommandError::ZeroQuantity),
            },
            Case {
                name: "invalid: iceberg with immediate TIF",
                order: LimitOrder::new(
                    1,
                    LimitOrderSpec::new(
                        100,
                        QuantityPolicy::Standard { quantity: 10 },
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    ),
                ),
                patch: LimitOrderPatch::new()
                    .with_quantity_policy(QuantityPolicy::Iceberg {
                        visible_quantity: 10,
                        hidden_quantity: 10,
                        replenish_quantity: 10,
                    })
                    .with_time_in_force(TimeInForce::Ioc),
                expected: Err(CommandError::IcebergImmediateTif),
            },
            Case {
                name: "valid patch + valid order → invalid: order is post_only, patch sets immediate TIF",
                order: LimitOrder::new(
                    1,
                    LimitOrderSpec::new(
                        100,
                        QuantityPolicy::Standard { quantity: 10 },
                        OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                    ),
                ),
                patch: LimitOrderPatch::new().with_time_in_force(TimeInForce::Ioc),
                expected: Err(CommandError::PostOnlyImmediateTif),
            },
            Case {
                name: "valid patch + valid order → invalid: order is immediate TIF, patch sets post_only",
                order: LimitOrder::new(
                    1,
                    LimitOrderSpec::new(
                        100,
                        QuantityPolicy::Standard { quantity: 10 },
                        OrderFlags::new(Side::Buy, false, TimeInForce::Ioc),
                    ),
                ),
                patch: LimitOrderPatch::new().with_post_only(true),
                expected: Err(CommandError::PostOnlyImmediateTif),
            },
            Case {
                name: "valid patch + valid order → invalid: order is iceberg, patch sets immediate TIF",
                order: LimitOrder::new(
                    1,
                    LimitOrderSpec::new(
                        100,
                        QuantityPolicy::Iceberg {
                            visible_quantity: 10,
                            hidden_quantity: 10,
                            replenish_quantity: 10,
                        },
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    ),
                ),
                patch: LimitOrderPatch::new().with_time_in_force(TimeInForce::Ioc),
                expected: Err(CommandError::IcebergImmediateTif),
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
    fn test_apply_pegged_patch() {
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
                    1,
                    PeggedOrderSpec::new(
                        PegReference::Market,
                        10,
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    ),
                ),
                patch: PeggedOrderPatch::new()
                    .with_peg_reference(PegReference::Market)
                    .with_quantity(10),
                expected: Ok(()),
            },
            Case {
                name: "update peg_reference only",
                order: PeggedOrder::new(
                    1,
                    PeggedOrderSpec::new(
                        PegReference::Primary,
                        10,
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    ),
                ),
                patch: PeggedOrderPatch::new().with_peg_reference(PegReference::Market),
                expected: Ok(()),
            },
            Case {
                name: "update quantity only",
                order: PeggedOrder::new(
                    1,
                    PeggedOrderSpec::new(
                        PegReference::Market,
                        10,
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    ),
                ),
                patch: PeggedOrderPatch::new().with_quantity(20),
                expected: Ok(()),
            },
            Case {
                name: "update post_only only",
                order: PeggedOrder::new(
                    1,
                    PeggedOrderSpec::new(
                        PegReference::Market,
                        10,
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    ),
                ),
                patch: PeggedOrderPatch::new().with_post_only(true),
                expected: Ok(()),
            },
            Case {
                name: "update time_in_force only",
                order: PeggedOrder::new(
                    1,
                    PeggedOrderSpec::new(
                        PegReference::Market,
                        10,
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    ),
                ),
                patch: PeggedOrderPatch::new().with_time_in_force(TimeInForce::Ioc),
                expected: Ok(()),
            },
            Case {
                name: "invalid: empty patch",
                order: PeggedOrder::new(
                    1,
                    PeggedOrderSpec::new(
                        PegReference::Market,
                        10,
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    ),
                ),
                patch: PeggedOrderPatch::new(),
                expected: Err(CommandError::EmptyPatch),
            },
            Case {
                name: "invalid: post-only with immediate TIF",
                order: PeggedOrder::new(
                    1,
                    PeggedOrderSpec::new(
                        PegReference::Market,
                        10,
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    ),
                ),
                patch: PeggedOrderPatch::new()
                    .with_post_only(true)
                    .with_time_in_force(TimeInForce::Ioc),
                expected: Err(CommandError::PostOnlyImmediateTif),
            },
            Case {
                name: "invalid: zero quantity",
                order: PeggedOrder::new(
                    1,
                    PeggedOrderSpec::new(
                        PegReference::Market,
                        10,
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    ),
                ),
                patch: PeggedOrderPatch::new().with_quantity(0),
                expected: Err(CommandError::ZeroQuantity),
            },
            Case {
                name: "invalid: peg reference Primary + immediate TIF",
                order: PeggedOrder::new(
                    1,
                    PeggedOrderSpec::new(
                        PegReference::Market,
                        10,
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    ),
                ),
                patch: PeggedOrderPatch::new()
                    .with_peg_reference(PegReference::Primary)
                    .with_time_in_force(TimeInForce::Ioc),
                expected: Err(CommandError::PeggedNonTakerImmediateTif),
            },
            Case {
                name: "valid patch + valid order → invalid: order is post_only, patch sets immediate TIF",
                order: PeggedOrder::new(
                    1,
                    PeggedOrderSpec::new(
                        PegReference::Primary,
                        10,
                        OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                    ),
                ),
                patch: PeggedOrderPatch::new().with_time_in_force(TimeInForce::Ioc),
                expected: Err(CommandError::PostOnlyImmediateTif),
            },
            Case {
                name: "valid patch + valid order → invalid: order is Primary, patch sets immediate TIF",
                order: PeggedOrder::new(
                    1,
                    PeggedOrderSpec::new(
                        PegReference::Primary,
                        10,
                        OrderFlags::new(Side::Buy, false, TimeInForce::Ioc),
                    ),
                ),
                patch: PeggedOrderPatch::new().with_time_in_force(TimeInForce::Ioc),
                expected: Err(CommandError::PeggedNonTakerImmediateTif),
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

    #[test]
    fn test_has_expired_time_in_force() {
        struct Case {
            name: &'static str,
            patch: OrderFlagsPatch,
            timestamp: u64,
            expected: bool,
        }

        let cases = [
            Case {
                name: "empty patch does not have expired time in force",
                patch: OrderFlagsPatch::default(),
                timestamp: 1000,
                expected: false,
            },
            Case {
                name: "GTC patch does not have expired time in force",
                patch: OrderFlagsPatch {
                    post_only: None,
                    time_in_force: Some(TimeInForce::Gtc),
                },
                timestamp: 1000,
                expected: false,
            },
            Case {
                name: "IOC patch does not have expired time in force",
                patch: OrderFlagsPatch {
                    post_only: None,
                    time_in_force: Some(TimeInForce::Ioc),
                },
                timestamp: 1000,
                expected: false,
            },
            Case {
                name: "FOK patch does not have expired time in force",
                patch: OrderFlagsPatch {
                    post_only: None,
                    time_in_force: Some(TimeInForce::Fok),
                },
                timestamp: 1000,
                expected: false,
            },
            Case {
                name: "GTD patch does not have expired time in force",
                patch: OrderFlagsPatch {
                    post_only: None,
                    time_in_force: Some(TimeInForce::Gtd(1000)),
                },
                timestamp: 999,
                expected: false,
            },
            Case {
                name: "GTD order has expired time in force",
                patch: OrderFlagsPatch {
                    post_only: None,
                    time_in_force: Some(TimeInForce::Gtd(1000)),
                },
                timestamp: 1000,
                expected: true,
            },
        ];

        for case in cases {
            let result = case.patch.has_expired_time_in_force(case.timestamp);
            assert_eq!(result, case.expected, "case: {}", case.name);
        }
    }
}

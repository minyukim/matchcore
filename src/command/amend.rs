use crate::{
    LimitOrder, PegReference, PeggedOrder, QuantityPolicy, TimeInForce,
    command::{
        CommandError, NewOrderCore,
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
    Limit(LimitPatch),
    /// The patch to a pegged order
    Pegged(PeggedPatch),
}

/// Represents the patch to a limit order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LimitPatch {
    /// The core patch
    pub core: PatchCore,
    /// The new price of the order
    pub new_price: Option<u64>,
    /// The new quantity policy of the order
    pub new_quantity_policy: Option<QuantityPolicy>,
}

impl LimitPatch {
    /// Create a new empty limit patch
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns this limit patch with the price set.
    pub fn with_price(mut self, v: u64) -> Self {
        self.new_price = Some(v);
        self
    }

    /// Returns this limit patch with the quantity policy set.
    pub fn with_quantity_policy(mut self, v: QuantityPolicy) -> Self {
        self.new_quantity_policy = Some(v);
        self
    }

    /// Returns this limit patch with the post-only set.
    pub fn with_post_only(mut self, v: bool) -> Self {
        self.core.new_post_only = Some(v);
        self
    }

    /// Returns this limit patch with the time-in-force set.
    pub fn with_time_in_force(mut self, v: TimeInForce) -> Self {
        self.core.new_time_in_force = Some(v);
        self
    }

    /// Check if the patch is empty
    pub fn is_empty(&self) -> bool {
        self.core.is_empty() && self.new_price.is_none() && self.new_quantity_policy.is_none()
    }

    /// Apply the patch to the order if the patch does not conflict with the order
    #[allow(unused)]
    pub(crate) fn apply(&self, order: &mut LimitOrder) -> Result<(), CommandError> {
        if self.is_empty() {
            return Err(CommandError::EmptyPatch);
        }

        let new_post_only = self.core.new_post_only.unwrap_or(order.is_post_only());
        let new_time_in_force = self.core.new_time_in_force.unwrap_or(order.time_in_force());
        let new_price = self.new_price.unwrap_or(order.price());
        let new_quantity_policy = self.new_quantity_policy.unwrap_or(order.quantity_policy());

        let new_core = NewOrderCore {
            side: order.side(),
            post_only: new_post_only,
            time_in_force: new_time_in_force,
        };
        validate_limit_order_invariants(&new_core, new_price, new_quantity_policy)?;

        order.update_post_only(new_post_only);
        order.update_time_in_force(new_time_in_force);
        order.update_price(new_price);
        order.update_quantity_policy(new_quantity_policy);

        Ok(())
    }
}

/// Represents the patch to a pegged order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PeggedPatch {
    /// The core patch
    pub core: PatchCore,
    /// The new peg reference type
    pub new_peg_reference: Option<PegReference>,
    /// The new quantity of the order
    pub new_quantity: Option<u64>,
}

impl PeggedPatch {
    /// Create a new empty pegged patch
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns this pegged patch with the peg reference set.
    pub fn with_peg_reference(mut self, v: PegReference) -> Self {
        self.new_peg_reference = Some(v);
        self
    }

    /// Returns this pegged patch with the quantity set.
    pub fn with_quantity(mut self, v: u64) -> Self {
        self.new_quantity = Some(v);
        self
    }

    /// Returns this pegged patch with the post-only set.
    pub fn with_post_only(mut self, v: bool) -> Self {
        self.core.new_post_only = Some(v);
        self
    }

    /// Returns this pegged patch with the time-in-force set.
    pub fn with_time_in_force(mut self, v: TimeInForce) -> Self {
        self.core.new_time_in_force = Some(v);
        self
    }

    /// Check if the patch is empty
    pub fn is_empty(&self) -> bool {
        self.core.is_empty() && self.new_peg_reference.is_none() && self.new_quantity.is_none()
    }

    /// Apply the patch to the order if the patch does not conflict with the order
    #[allow(unused)]
    pub(crate) fn apply(&self, order: &mut PeggedOrder) -> Result<(), CommandError> {
        if self.is_empty() {
            return Err(CommandError::EmptyPatch);
        }

        let new_post_only = self.core.new_post_only.unwrap_or(order.is_post_only());
        let new_time_in_force = self.core.new_time_in_force.unwrap_or(order.time_in_force());
        let new_peg_reference = self.new_peg_reference.unwrap_or(order.peg_reference());
        let new_quantity = self.new_quantity.unwrap_or(order.quantity());

        let new_core = NewOrderCore {
            side: order.side(),
            post_only: new_post_only,
            time_in_force: new_time_in_force,
        };
        validate_pegged_order_invariants(&new_core, new_peg_reference, new_quantity)?;

        order.update_post_only(new_post_only);
        order.update_time_in_force(new_time_in_force);
        order.update_peg_reference(new_peg_reference);
        order.update_quantity(new_quantity);

        Ok(())
    }
}

/// Represents the shared core patch for all order types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PatchCore {
    /// The new post-only flag
    pub new_post_only: Option<bool>,
    /// The new time in force of the order
    pub new_time_in_force: Option<TimeInForce>,
}

impl PatchCore {
    /// Validate the patch core
    pub fn validate(&self) -> Result<(), CommandError> {
        if let Some(time_in_force) = self.new_time_in_force
            && let Some(post_only) = self.new_post_only
            && time_in_force.is_immediate()
            && post_only
        {
            return Err(CommandError::PostOnlyImmediateTif);
        }
        Ok(())
    }

    /// Check if the patch core is empty
    pub fn is_empty(&self) -> bool {
        self.new_post_only.is_none() && self.new_time_in_force.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{OrderCore, Side};

    #[test]
    fn test_apply_limit_patch() {
        struct Case {
            name: &'static str,
            order: LimitOrder,
            patch: LimitPatch,
            expected: Result<(), CommandError>,
        }

        let cases = [
            Case {
                name: "no-op patch (same price and quantity policy)",
                order: LimitOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Gtc),
                    100,
                    QuantityPolicy::Standard { quantity: 10 },
                ),
                patch: LimitPatch::new()
                    .with_price(100)
                    .with_quantity_policy(QuantityPolicy::Standard { quantity: 10 }),
                expected: Ok(()),
            },
            Case {
                name: "update price only",
                order: LimitOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Gtc),
                    100,
                    QuantityPolicy::Standard { quantity: 10 },
                ),
                patch: LimitPatch::new().with_price(200),
                expected: Ok(()),
            },
            Case {
                name: "update quantity policy only",
                order: LimitOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Gtc),
                    100,
                    QuantityPolicy::Standard { quantity: 10 },
                ),
                patch: LimitPatch::new()
                    .with_quantity_policy(QuantityPolicy::Standard { quantity: 20 }),
                expected: Ok(()),
            },
            Case {
                name: "update post_only only",
                order: LimitOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Gtc),
                    100,
                    QuantityPolicy::Standard { quantity: 10 },
                ),
                patch: LimitPatch::new().with_post_only(true),
                expected: Ok(()),
            },
            Case {
                name: "update time_in_force only",
                order: LimitOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Gtc),
                    100,
                    QuantityPolicy::Standard { quantity: 10 },
                ),
                patch: LimitPatch::new().with_time_in_force(TimeInForce::Ioc),
                expected: Ok(()),
            },
            Case {
                name: "invalid: empty patch",
                order: LimitOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Gtc),
                    100,
                    QuantityPolicy::Standard { quantity: 10 },
                ),
                patch: LimitPatch::new(),
                expected: Err(CommandError::EmptyPatch),
            },
            Case {
                name: "invalid: post-only with immediate TIF",
                order: LimitOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Gtc),
                    100,
                    QuantityPolicy::Standard { quantity: 10 },
                ),
                patch: LimitPatch::new()
                    .with_post_only(true)
                    .with_time_in_force(TimeInForce::Ioc),
                expected: Err(CommandError::PostOnlyImmediateTif),
            },
            Case {
                name: "invalid: zero price",
                order: LimitOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Gtc),
                    100,
                    QuantityPolicy::Standard { quantity: 10 },
                ),
                patch: LimitPatch::new().with_price(0),
                expected: Err(CommandError::ZeroPrice),
            },
            Case {
                name: "invalid: zero quantity",
                order: LimitOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Gtc),
                    100,
                    QuantityPolicy::Standard { quantity: 10 },
                ),
                patch: LimitPatch::new()
                    .with_price(100)
                    .with_quantity_policy(QuantityPolicy::Standard { quantity: 0 }),
                expected: Err(CommandError::ZeroQuantity),
            },
            Case {
                name: "valid patch + valid order → invalid: order is post_only, patch sets immediate TIF",
                order: LimitOrder::new(
                    OrderCore::new(1, Side::Buy, true, TimeInForce::Gtc),
                    100,
                    QuantityPolicy::Standard { quantity: 10 },
                ),
                patch: LimitPatch::new().with_time_in_force(TimeInForce::Ioc),
                expected: Err(CommandError::PostOnlyImmediateTif),
            },
            Case {
                name: "valid patch + valid order → invalid: order is immediate TIF, patch sets post_only",
                order: LimitOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Ioc),
                    100,
                    QuantityPolicy::Standard { quantity: 10 },
                ),
                patch: LimitPatch::new().with_post_only(true),
                expected: Err(CommandError::PostOnlyImmediateTif),
            },
        ];

        for case in cases {
            let mut order = case.order.clone();
            let result = case.patch.apply(&mut order);

            match (&case.expected, &result) {
                (Ok(()), Ok(())) => {
                    // Verify order was updated as expected
                    let expected_price = case.patch.new_price.unwrap_or(case.order.price());
                    let expected_quantity_policy = case
                        .patch
                        .new_quantity_policy
                        .unwrap_or(case.order.quantity_policy());
                    let expected_post_only = case
                        .patch
                        .core
                        .new_post_only
                        .unwrap_or(case.order.is_post_only());
                    let expected_time_in_force = case
                        .patch
                        .core
                        .new_time_in_force
                        .unwrap_or(case.order.time_in_force());

                    assert_eq!(order.price(), expected_price, "case: {}", case.name);
                    assert_eq!(
                        order.quantity_policy(),
                        expected_quantity_policy,
                        "case: {}",
                        case.name
                    );
                    assert_eq!(
                        order.is_post_only(),
                        expected_post_only,
                        "case: {}",
                        case.name
                    );
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
            patch: PeggedPatch,
            expected: Result<(), CommandError>,
        }

        let cases = [
            Case {
                name: "no-op patch (same peg_reference and quantity)",
                order: PeggedOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Gtc),
                    PegReference::Market,
                    10,
                ),
                patch: PeggedPatch::new()
                    .with_peg_reference(PegReference::Market)
                    .with_quantity(10),
                expected: Ok(()),
            },
            Case {
                name: "update peg_reference only",
                order: PeggedOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Gtc),
                    PegReference::Primary,
                    10,
                ),
                patch: PeggedPatch::new().with_peg_reference(PegReference::Market),
                expected: Ok(()),
            },
            Case {
                name: "update quantity only",
                order: PeggedOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Gtc),
                    PegReference::Market,
                    10,
                ),
                patch: PeggedPatch::new().with_quantity(20),
                expected: Ok(()),
            },
            Case {
                name: "update post_only only",
                order: PeggedOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Gtc),
                    PegReference::Market,
                    10,
                ),
                patch: PeggedPatch::new().with_post_only(true),
                expected: Ok(()),
            },
            Case {
                name: "update time_in_force only",
                order: PeggedOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Gtc),
                    PegReference::Market,
                    10,
                ),
                patch: PeggedPatch::new().with_time_in_force(TimeInForce::Ioc),
                expected: Ok(()),
            },
            Case {
                name: "invalid: empty patch",
                order: PeggedOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Gtc),
                    PegReference::Market,
                    10,
                ),
                patch: PeggedPatch::new(),
                expected: Err(CommandError::EmptyPatch),
            },
            Case {
                name: "invalid: post-only with immediate TIF",
                order: PeggedOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Gtc),
                    PegReference::Market,
                    10,
                ),
                patch: PeggedPatch::new()
                    .with_post_only(true)
                    .with_time_in_force(TimeInForce::Ioc),
                expected: Err(CommandError::PostOnlyImmediateTif),
            },
            Case {
                name: "invalid: zero quantity",
                order: PeggedOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Gtc),
                    PegReference::Market,
                    10,
                ),
                patch: PeggedPatch::new().with_quantity(0),
                expected: Err(CommandError::ZeroQuantity),
            },
            Case {
                name: "valid patch + valid order → invalid: peg reference Primary + immediate TIF",
                order: PeggedOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Gtc),
                    PegReference::Primary,
                    10,
                ),
                patch: PeggedPatch::new()
                    .with_peg_reference(PegReference::Primary)
                    .with_time_in_force(TimeInForce::Ioc),
                expected: Err(CommandError::PeggedNonTakerImmediateTif),
            },
            Case {
                name: "valid patch + valid order → invalid: order is post_only, patch sets immediate TIF",
                order: PeggedOrder::new(
                    OrderCore::new(1, Side::Buy, true, TimeInForce::Gtc),
                    PegReference::Market,
                    10,
                ),
                patch: PeggedPatch::new().with_time_in_force(TimeInForce::Ioc),
                expected: Err(CommandError::PostOnlyImmediateTif),
            },
            Case {
                name: "valid patch + valid order → invalid: order is immediate TIF, patch sets post_only",
                order: PeggedOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Ioc),
                    PegReference::Market,
                    10,
                ),
                patch: PeggedPatch::new().with_post_only(true),
                expected: Err(CommandError::PostOnlyImmediateTif),
            },
        ];

        for case in cases {
            let mut order = case.order.clone();
            let result = case.patch.apply(&mut order);

            match (&case.expected, &result) {
                (Ok(()), Ok(())) => {
                    let expected_peg_reference = case
                        .patch
                        .new_peg_reference
                        .unwrap_or(case.order.peg_reference());
                    let expected_quantity =
                        case.patch.new_quantity.unwrap_or(case.order.quantity());
                    let expected_post_only = case
                        .patch
                        .core
                        .new_post_only
                        .unwrap_or(case.order.is_post_only());
                    let expected_time_in_force = case
                        .patch
                        .core
                        .new_time_in_force
                        .unwrap_or(case.order.time_in_force());

                    assert_eq!(
                        order.peg_reference(),
                        expected_peg_reference,
                        "case: {}",
                        case.name
                    );
                    assert_eq!(order.quantity(), expected_quantity, "case: {}", case.name);
                    assert_eq!(
                        order.is_post_only(),
                        expected_post_only,
                        "case: {}",
                        case.name
                    );
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
}

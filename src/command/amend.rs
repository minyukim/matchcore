use crate::{
    LimitOrder, PegReference, QuantityPolicy, TimeInForce,
    command::{CommandError, NewOrderCore, validate_limit_order_invariants},
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LimitPatch {
    /// The core patch
    pub core: PatchCore,
    /// The new price of the order
    pub new_price: Option<u64>,
    /// The new quantity policy of the order
    pub new_quantity_policy: Option<QuantityPolicy>,
}

impl LimitPatch {
    /// Validate the patch
    pub fn validate(&self) -> Result<(), CommandError> {
        if self.is_empty() {
            return Err(CommandError::EmptyPatch);
        }

        self.core.validate()?;

        if let Some(price) = self.new_price
            && price == 0
        {
            return Err(CommandError::ZeroPrice);
        }

        if let Some(quantity_policy) = self.new_quantity_policy {
            match quantity_policy {
                QuantityPolicy::Standard { quantity } => {
                    if quantity == 0 {
                        return Err(CommandError::ZeroQuantity);
                    }
                }
                QuantityPolicy::Iceberg {
                    visible_quantity,
                    hidden_quantity,
                    replenish_quantity,
                } => {
                    if visible_quantity == 0 {
                        return Err(CommandError::ZeroQuantity);
                    }
                    if hidden_quantity == 0 {
                        return Err(CommandError::IcebergZeroHiddenQuantity);
                    }
                    if replenish_quantity == 0 {
                        return Err(CommandError::IcebergZeroReplenishQuantity);
                    }
                }
            }
        };
        Ok(())
    }

    /// Check if the patch is empty
    pub fn is_empty(&self) -> bool {
        self.core.is_empty() && self.new_price.is_none() && self.new_quantity_policy.is_none()
    }

    /// Apply the patch to the order if the patch does not conflict with the order
    #[allow(unused)]
    pub(crate) fn apply(&self, order: &mut LimitOrder) -> Result<(), CommandError> {
        let new_post_only = self.core.new_post_only.unwrap_or(order.is_post_only());
        let new_time_in_force = self.core.new_time_in_force.unwrap_or(order.time_in_force());
        let new_price = self.new_price.unwrap_or(order.price());
        let new_quantity_policy = self.new_quantity_policy.unwrap_or(order.quantity_policy());

        let new_core = NewOrderCore {
            side: order.side(),
            post_only: new_post_only,
            time_in_force: new_time_in_force,
            extra: (),
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeggedPatch {
    /// The core patch
    pub core: PatchCore,
    /// The new peg reference type
    pub new_peg_reference: Option<PegReference>,
    /// The new quantity of the order
    pub new_quantity: Option<u64>,
}

impl PeggedPatch {
    /// Validate the patch
    pub fn validate(&self) -> Result<(), CommandError> {
        if self.is_empty() {
            return Err(CommandError::EmptyPatch);
        }

        self.core.validate()?;

        if let Some(quantity) = self.new_quantity
            && quantity == 0
        {
            return Err(CommandError::ZeroQuantity);
        }
        if let Some(peg_reference) = self.new_peg_reference
            && let Some(time_in_force) = self.core.new_time_in_force
            && !peg_reference.can_be_taker()
            && time_in_force.is_immediate()
        {
            return Err(CommandError::PeggedNonTakerImmediateTif);
        }
        Ok(())
    }

    /// Check if the patch is empty
    pub fn is_empty(&self) -> bool {
        self.core.is_empty() && self.new_peg_reference.is_none() && self.new_quantity.is_none()
    }
}

/// Represents the shared core patch for all order types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    fn test_validate_limit_patch() {
        struct Case {
            name: &'static str,
            patch: LimitPatch,
            expected: Result<(), CommandError>,
        }

        let cases = [
            Case {
                name: "empty limit patch",
                patch: LimitPatch {
                    core: PatchCore {
                        new_post_only: None,
                        new_time_in_force: None,
                    },
                    new_price: None,
                    new_quantity_policy: None,
                },
                expected: Err(CommandError::EmptyPatch),
            },
            Case {
                name: "valid limit patch",
                patch: LimitPatch {
                    core: PatchCore {
                        new_post_only: None,
                        new_time_in_force: None,
                    },
                    new_price: Some(100),
                    new_quantity_policy: None,
                },
                expected: Ok(()),
            },
            Case {
                name: "zero price",
                patch: LimitPatch {
                    core: PatchCore {
                        new_post_only: None,
                        new_time_in_force: None,
                    },
                    new_price: Some(0),
                    new_quantity_policy: None,
                },
                expected: Err(CommandError::ZeroPrice),
            },
            Case {
                name: "zero quantity standard limit patch",
                patch: LimitPatch {
                    core: PatchCore {
                        new_post_only: None,
                        new_time_in_force: None,
                    },
                    new_price: Some(100),
                    new_quantity_policy: Some(QuantityPolicy::Standard { quantity: 0 }),
                },
                expected: Err(CommandError::ZeroQuantity),
            },
            Case {
                name: "zero hidden quantity",
                patch: LimitPatch {
                    core: PatchCore {
                        new_post_only: None,
                        new_time_in_force: None,
                    },
                    new_price: None,
                    new_quantity_policy: Some(QuantityPolicy::Iceberg {
                        visible_quantity: 10,
                        hidden_quantity: 0,
                        replenish_quantity: 10,
                    }),
                },
                expected: Err(CommandError::IcebergZeroHiddenQuantity),
            },
            Case {
                name: "zero replenish quantity",
                patch: LimitPatch {
                    core: PatchCore {
                        new_post_only: None,
                        new_time_in_force: None,
                    },
                    new_price: None,
                    new_quantity_policy: Some(QuantityPolicy::Iceberg {
                        visible_quantity: 10,
                        hidden_quantity: 10,
                        replenish_quantity: 0,
                    }),
                },
                expected: Err(CommandError::IcebergZeroReplenishQuantity),
            },
            Case {
                name: "post-only limit patch",
                patch: LimitPatch {
                    core: PatchCore {
                        new_post_only: Some(true),
                        new_time_in_force: None,
                    },
                    new_price: None,
                    new_quantity_policy: None,
                },
                expected: Ok(()),
            },
            Case {
                name: "immediate time in force limit patch",
                patch: LimitPatch {
                    core: PatchCore {
                        new_post_only: None,
                        new_time_in_force: Some(TimeInForce::Ioc),
                    },
                    new_price: None,
                    new_quantity_policy: None,
                },
                expected: Ok(()),
            },
            Case {
                name: "post-only immediate time in force limit patch",
                patch: LimitPatch {
                    core: PatchCore {
                        new_post_only: Some(true),
                        new_time_in_force: Some(TimeInForce::Ioc),
                    },
                    new_price: None,
                    new_quantity_policy: None,
                },
                expected: Err(CommandError::PostOnlyImmediateTif),
            },
        ];

        for case in cases {
            let patch = case.patch;
            match case.expected {
                Ok(()) => assert!(patch.validate().is_ok(), "case: {}", case.name),
                Err(e) => assert_eq!(patch.validate().unwrap_err(), e, "case: {}", case.name),
            }
        }
    }

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
                name: "no-op patch (same price, no core or quantity change)",
                order: LimitOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Gtc, ()),
                    100,
                    QuantityPolicy::Standard { quantity: 10 },
                ),
                patch: LimitPatch {
                    core: PatchCore {
                        new_post_only: None,
                        new_time_in_force: None,
                    },
                    new_price: Some(100),
                    new_quantity_policy: None,
                },
                expected: Ok(()),
            },
            Case {
                name: "update price only",
                order: LimitOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Gtc, ()),
                    100,
                    QuantityPolicy::Standard { quantity: 10 },
                ),
                patch: LimitPatch {
                    core: PatchCore {
                        new_post_only: None,
                        new_time_in_force: None,
                    },
                    new_price: Some(200),
                    new_quantity_policy: None,
                },
                expected: Ok(()),
            },
            Case {
                name: "update quantity policy only",
                order: LimitOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Gtc, ()),
                    100,
                    QuantityPolicy::Standard { quantity: 10 },
                ),
                patch: LimitPatch {
                    core: PatchCore {
                        new_post_only: None,
                        new_time_in_force: None,
                    },
                    new_price: None,
                    new_quantity_policy: Some(QuantityPolicy::Standard { quantity: 20 }),
                },
                expected: Ok(()),
            },
            Case {
                name: "update post_only via core",
                order: LimitOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Gtc, ()),
                    100,
                    QuantityPolicy::Standard { quantity: 10 },
                ),
                patch: LimitPatch {
                    core: PatchCore {
                        new_post_only: Some(true),
                        new_time_in_force: None,
                    },
                    new_price: None,
                    new_quantity_policy: None,
                },
                expected: Ok(()),
            },
            Case {
                name: "update time_in_force via core",
                order: LimitOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Gtc, ()),
                    100,
                    QuantityPolicy::Standard { quantity: 10 },
                ),
                patch: LimitPatch {
                    core: PatchCore {
                        new_post_only: None,
                        new_time_in_force: Some(TimeInForce::Ioc),
                    },
                    new_price: None,
                    new_quantity_policy: None,
                },
                expected: Ok(()),
            },
            Case {
                name: "invalid: post-only with immediate TIF",
                order: LimitOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Gtc, ()),
                    100,
                    QuantityPolicy::Standard { quantity: 10 },
                ),
                patch: LimitPatch {
                    core: PatchCore {
                        new_post_only: Some(true),
                        new_time_in_force: Some(TimeInForce::Ioc),
                    },
                    new_price: None,
                    new_quantity_policy: None,
                },
                expected: Err(CommandError::PostOnlyImmediateTif),
            },
            Case {
                name: "invalid: zero price",
                order: LimitOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Gtc, ()),
                    100,
                    QuantityPolicy::Standard { quantity: 10 },
                ),
                patch: LimitPatch {
                    core: PatchCore {
                        new_post_only: None,
                        new_time_in_force: None,
                    },
                    new_price: Some(0),
                    new_quantity_policy: None,
                },
                expected: Err(CommandError::ZeroPrice),
            },
            Case {
                name: "invalid: zero quantity",
                order: LimitOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Gtc, ()),
                    100,
                    QuantityPolicy::Standard { quantity: 10 },
                ),
                patch: LimitPatch {
                    core: PatchCore {
                        new_post_only: None,
                        new_time_in_force: None,
                    },
                    new_price: Some(100),
                    new_quantity_policy: Some(QuantityPolicy::Standard { quantity: 0 }),
                },
                expected: Err(CommandError::ZeroQuantity),
            },
            Case {
                name: "valid patch + valid order → invalid: order is post_only, patch sets immediate TIF",
                order: LimitOrder::new(
                    OrderCore::new(1, Side::Buy, true, TimeInForce::Gtc, ()),
                    100,
                    QuantityPolicy::Standard { quantity: 10 },
                ),
                patch: LimitPatch {
                    core: PatchCore {
                        new_post_only: None,
                        new_time_in_force: Some(TimeInForce::Ioc),
                    },
                    new_price: None,
                    new_quantity_policy: None,
                },
                expected: Err(CommandError::PostOnlyImmediateTif),
            },
            Case {
                name: "valid patch + valid order → invalid: order is immediate TIF, patch sets post_only",
                order: LimitOrder::new(
                    OrderCore::new(1, Side::Buy, false, TimeInForce::Ioc, ()),
                    100,
                    QuantityPolicy::Standard { quantity: 10 },
                ),
                patch: LimitPatch {
                    core: PatchCore {
                        new_post_only: Some(true),
                        new_time_in_force: None,
                    },
                    new_price: None,
                    new_quantity_policy: None,
                },
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
    fn test_validate_pegged_patch() {
        struct Case {
            name: &'static str,
            patch: PeggedPatch,
            expected: Result<(), CommandError>,
        }

        let cases = [
            Case {
                name: "empty pegged patch",
                patch: PeggedPatch {
                    core: PatchCore {
                        new_post_only: None,
                        new_time_in_force: None,
                    },
                    new_peg_reference: None,
                    new_quantity: None,
                },
                expected: Err(CommandError::EmptyPatch),
            },
            Case {
                name: "valid pegged patch",
                patch: PeggedPatch {
                    core: PatchCore {
                        new_post_only: None,
                        new_time_in_force: None,
                    },
                    new_peg_reference: Some(PegReference::Market),
                    new_quantity: Some(100),
                },
                expected: Ok(()),
            },
            Case {
                name: "zero quantity",
                patch: PeggedPatch {
                    core: PatchCore {
                        new_post_only: None,
                        new_time_in_force: None,
                    },
                    new_peg_reference: None,
                    new_quantity: Some(0),
                },
                expected: Err(CommandError::ZeroQuantity),
            },
            Case {
                name: "post-only pegged patch",
                patch: PeggedPatch {
                    core: PatchCore {
                        new_post_only: Some(true),
                        new_time_in_force: None,
                    },
                    new_peg_reference: None,
                    new_quantity: Some(100),
                },
                expected: Ok(()),
            },
            Case {
                name: "immediate time in force pegged order",
                patch: PeggedPatch {
                    core: PatchCore {
                        new_post_only: None,
                        new_time_in_force: Some(TimeInForce::Ioc),
                    },
                    new_peg_reference: None,
                    new_quantity: Some(100),
                },
                expected: Ok(()),
            },
            Case {
                name: "post-only immediate time in force",
                patch: PeggedPatch {
                    core: PatchCore {
                        new_post_only: Some(true),
                        new_time_in_force: Some(TimeInForce::Ioc),
                    },
                    new_peg_reference: None,
                    new_quantity: Some(100),
                },
                expected: Err(CommandError::PostOnlyImmediateTif),
            },
            Case {
                name: "maker only immediate time in force",
                patch: PeggedPatch {
                    core: PatchCore {
                        new_post_only: None,
                        new_time_in_force: Some(TimeInForce::Ioc),
                    },
                    new_peg_reference: Some(PegReference::Primary),
                    new_quantity: Some(100),
                },
                expected: Err(CommandError::PeggedNonTakerImmediateTif),
            },
        ];

        for case in cases {
            let patch = case.patch;
            match case.expected {
                Ok(()) => assert!(patch.validate().is_ok(), "case: {}", case.name),
                Err(e) => assert_eq!(patch.validate().unwrap_err(), e, "case: {}", case.name),
            }
        }
    }
}

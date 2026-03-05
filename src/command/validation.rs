use super::CommandError;
use crate::{PegReference, Price, Quantity, QuantityPolicy, TimeInForce, orders::*};

impl MarketOrder {
    /// Validate the order specification
    pub fn validate(&self) -> Result<(), CommandError> {
        if self.quantity().is_zero() {
            return Err(CommandError::ZeroQuantity);
        }
        Ok(())
    }
}

impl LimitOrder {
    /// Validate the order specification
    pub fn validate(&self) -> Result<(), CommandError> {
        validate_limit_order_invariants(
            self.price(),
            self.quantity_policy(),
            self.post_only(),
            self.time_in_force(),
        )
    }
}

impl PeggedOrder {
    /// Validate the order specification
    pub fn validate(&self) -> Result<(), CommandError> {
        validate_pegged_order_invariants(
            self.peg_reference(),
            self.quantity(),
            self.post_only(),
            self.time_in_force(),
        )
    }
}

/// Validate the invariants of a limit order
pub(super) fn validate_limit_order_invariants(
    price: Price,
    quantity_policy: QuantityPolicy,
    post_only: bool,
    time_in_force: TimeInForce,
) -> Result<(), CommandError> {
    validate_order_core_invariants(post_only, time_in_force)?;

    if price.is_zero() {
        return Err(CommandError::ZeroPrice);
    }

    match quantity_policy {
        QuantityPolicy::Standard { quantity } => {
            if quantity.is_zero() {
                return Err(CommandError::ZeroQuantity);
            }
        }
        QuantityPolicy::Iceberg {
            visible_quantity,
            hidden_quantity,
            replenish_quantity,
        } => {
            if visible_quantity.is_zero() {
                return Err(CommandError::ZeroQuantity);
            }
            if hidden_quantity.is_zero() {
                return Err(CommandError::IcebergZeroHiddenQuantity);
            }
            if replenish_quantity.is_zero() {
                return Err(CommandError::IcebergZeroReplenishQuantity);
            }

            if time_in_force.is_immediate() {
                return Err(CommandError::IcebergImmediateTif);
            }
        }
    }
    Ok(())
}

/// Validate the invariants of a pegged order
pub(super) fn validate_pegged_order_invariants(
    peg_reference: PegReference,
    quantity: Quantity,
    post_only: bool,
    time_in_force: TimeInForce,
) -> Result<(), CommandError> {
    validate_order_core_invariants(post_only, time_in_force)?;

    if quantity.is_zero() {
        return Err(CommandError::ZeroQuantity);
    }

    if peg_reference.is_always_taker() {
        if post_only {
            return Err(CommandError::PeggedAlwaysTakerPostOnly);
        }
    } else if !peg_reference.can_be_taker() && time_in_force.is_immediate() {
        return Err(CommandError::PeggedNonTakerImmediateTif);
    }

    Ok(())
}

/// Validate the invariants of an order core
fn validate_order_core_invariants(
    post_only: bool,
    time_in_force: TimeInForce,
) -> Result<(), CommandError> {
    if post_only && time_in_force.is_immediate() {
        return Err(CommandError::PostOnlyImmediateTif);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Side;

    #[test]
    fn test_validate_market_order_spec() {
        struct Case {
            name: &'static str,
            spec: MarketOrder,
            expected: Result<(), CommandError>,
        }

        let cases = [
            Case {
                name: "valid market order",
                spec: MarketOrder::new(Quantity(100), Side::Buy, true),
                expected: Ok(()),
            },
            Case {
                name: "zero quantity",
                spec: MarketOrder::new(Quantity(0), Side::Buy, true),
                expected: Err(CommandError::ZeroQuantity),
            },
        ];

        for case in cases {
            match case.expected {
                Ok(()) => assert!(case.spec.validate().is_ok(), "case: {}", case.name),
                Err(e) => assert_eq!(case.spec.validate().unwrap_err(), e, "case: {}", case.name),
            }
        }
    }

    #[test]
    fn test_validate_limit_order_invariants() {
        struct Case {
            name: &'static str,
            price: Price,
            quantity_policy: QuantityPolicy,
            post_only: bool,
            time_in_force: TimeInForce,
            expected: Result<(), CommandError>,
        }

        let cases = [
            Case {
                name: "valid standard order",
                price: Price(100),
                quantity_policy: QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                post_only: false,
                time_in_force: TimeInForce::Gtc,
                expected: Ok(()),
            },
            Case {
                name: "zero price",
                price: Price(0),
                quantity_policy: QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                post_only: false,
                time_in_force: TimeInForce::Gtc,
                expected: Err(CommandError::ZeroPrice),
            },
            Case {
                name: "standard zero quantity",
                price: Price(100),
                quantity_policy: QuantityPolicy::Standard {
                    quantity: Quantity(0),
                },
                post_only: false,
                time_in_force: TimeInForce::Gtc,
                expected: Err(CommandError::ZeroQuantity),
            },
            Case {
                name: "iceberg zero visible quantity",
                price: Price(100),
                quantity_policy: QuantityPolicy::Iceberg {
                    visible_quantity: Quantity(0),
                    hidden_quantity: Quantity(10),
                    replenish_quantity: Quantity(10),
                },
                post_only: false,
                time_in_force: TimeInForce::Gtc,
                expected: Err(CommandError::ZeroQuantity),
            },
            Case {
                name: "iceberg zero hidden quantity",
                price: Price(100),
                quantity_policy: QuantityPolicy::Iceberg {
                    visible_quantity: Quantity(10),
                    hidden_quantity: Quantity(0),
                    replenish_quantity: Quantity(10),
                },
                post_only: false,
                time_in_force: TimeInForce::Gtc,
                expected: Err(CommandError::IcebergZeroHiddenQuantity),
            },
            Case {
                name: "iceberg zero replenish quantity",
                price: Price(100),
                quantity_policy: QuantityPolicy::Iceberg {
                    visible_quantity: Quantity(10),
                    hidden_quantity: Quantity(10),
                    replenish_quantity: Quantity(0),
                },
                post_only: false,
                time_in_force: TimeInForce::Gtc,
                expected: Err(CommandError::IcebergZeroReplenishQuantity),
            },
            Case {
                name: "post-only standard order",
                price: Price(100),
                quantity_policy: QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                post_only: true,
                time_in_force: TimeInForce::Gtc,
                expected: Ok(()),
            },
            Case {
                name: "immediate time in force standard order",
                price: Price(100),
                quantity_policy: QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                post_only: false,
                time_in_force: TimeInForce::Ioc,
                expected: Ok(()),
            },
            Case {
                name: "post-only immediate time in force",
                price: Price(100),
                quantity_policy: QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                post_only: true,
                time_in_force: TimeInForce::Ioc,
                expected: Err(CommandError::PostOnlyImmediateTif),
            },
            Case {
                name: "iceberg with immediate time in force",
                price: Price(100),
                quantity_policy: QuantityPolicy::Iceberg {
                    visible_quantity: Quantity(10),
                    hidden_quantity: Quantity(10),
                    replenish_quantity: Quantity(10),
                },
                post_only: false,
                time_in_force: TimeInForce::Ioc,
                expected: Err(CommandError::IcebergImmediateTif),
            },
        ];

        for case in cases {
            match case.expected {
                Ok(()) => assert!(
                    validate_limit_order_invariants(
                        case.price,
                        case.quantity_policy,
                        case.post_only,
                        case.time_in_force
                    )
                    .is_ok(),
                    "case: {}",
                    case.name
                ),
                Err(e) => assert_eq!(
                    validate_limit_order_invariants(
                        case.price,
                        case.quantity_policy,
                        case.post_only,
                        case.time_in_force
                    )
                    .unwrap_err(),
                    e,
                    "case: {}",
                    case.name
                ),
            }
        }
    }

    #[test]
    fn test_validate_pegged_order_invariants() {
        struct Case {
            name: &'static str,
            peg_reference: PegReference,
            quantity: Quantity,
            post_only: bool,
            time_in_force: TimeInForce,
            expected: Result<(), CommandError>,
        }
        let cases = [
            Case {
                name: "valid pegged order",
                peg_reference: PegReference::Market,
                quantity: Quantity(100),
                post_only: false,
                time_in_force: TimeInForce::Gtc,
                expected: Ok(()),
            },
            Case {
                name: "zero quantity",
                peg_reference: PegReference::Market,
                quantity: Quantity(0),
                post_only: false,
                time_in_force: TimeInForce::Gtc,
                expected: Err(CommandError::ZeroQuantity),
            },
            Case {
                name: "post-only pegged order",
                peg_reference: PegReference::Primary,
                quantity: Quantity(100),
                post_only: true,
                time_in_force: TimeInForce::Gtc,
                expected: Ok(()),
            },
            Case {
                name: "immediate time in force pegged order",
                peg_reference: PegReference::Market,
                quantity: Quantity(100),
                post_only: false,
                time_in_force: TimeInForce::Ioc,
                expected: Ok(()),
            },
            Case {
                name: "post-only immediate time in force",
                peg_reference: PegReference::Market,
                quantity: Quantity(100),
                post_only: true,
                time_in_force: TimeInForce::Ioc,
                expected: Err(CommandError::PostOnlyImmediateTif),
            },
            Case {
                name: "maker only immediate time in force",
                peg_reference: PegReference::Primary,
                quantity: Quantity(100),
                post_only: false,
                time_in_force: TimeInForce::Ioc,
                expected: Err(CommandError::PeggedNonTakerImmediateTif),
            },
            Case {
                name: "always taker post-only",
                peg_reference: PegReference::Market,
                quantity: Quantity(100),
                post_only: true,
                time_in_force: TimeInForce::Gtc,
                expected: Err(CommandError::PeggedAlwaysTakerPostOnly),
            },
        ];

        for case in cases {
            match case.expected {
                Ok(()) => assert!(
                    validate_pegged_order_invariants(
                        case.peg_reference,
                        case.quantity,
                        case.post_only,
                        case.time_in_force
                    )
                    .is_ok(),
                    "case: {}",
                    case.name
                ),
                Err(e) => assert_eq!(
                    validate_pegged_order_invariants(
                        case.peg_reference,
                        case.quantity,
                        case.post_only,
                        case.time_in_force
                    )
                    .unwrap_err(),
                    e,
                    "case: {}",
                    case.name
                ),
            }
        }
    }
}

use crate::{PegReference, QuantityPolicy, TimeInForce, command::CommandError, orders::*};

impl MarketOrderSpec {
    /// Validate the order
    pub fn validate(&self) -> Result<(), CommandError> {
        if self.quantity() == 0 {
            return Err(CommandError::ZeroQuantity);
        }
        Ok(())
    }
}

/// Validate the invariants of a limit order
pub(super) fn validate_limit_order_invariants(
    price: u64,
    quantity_policy: QuantityPolicy,
    post_only: bool,
    time_in_force: TimeInForce,
) -> Result<(), CommandError> {
    validate_order_core_invariants(post_only, time_in_force)?;

    if price == 0 {
        return Err(CommandError::ZeroPrice);
    }

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
    quantity: u64,
    post_only: bool,
    time_in_force: TimeInForce,
) -> Result<(), CommandError> {
    validate_order_core_invariants(post_only, time_in_force)?;

    if quantity == 0 {
        return Err(CommandError::ZeroQuantity);
    }

    if !peg_reference.can_be_taker() && time_in_force.is_immediate() {
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
            spec: MarketOrderSpec,
            expected: Result<(), CommandError>,
        }

        let cases = [
            Case {
                name: "valid market order",
                spec: MarketOrderSpec::new(100, Side::Buy, true),
                expected: Ok(()),
            },
            Case {
                name: "zero quantity",
                spec: MarketOrderSpec::new(0, Side::Buy, true),
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
}

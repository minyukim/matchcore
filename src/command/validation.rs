use crate::{PegReference, QuantityPolicy, TimeInForce, command::CommandError};

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

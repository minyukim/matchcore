use crate::{
    PegReference, QuantityPolicy,
    command::{CommandError, NewOrderCore},
};

/// Validate the invariants of a limit order
pub(super) fn validate_limit_order_invariants(
    core: &NewOrderCore,
    price: u64,
    quantity_policy: QuantityPolicy,
) -> Result<(), CommandError> {
    core.validate()?;

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
    core: &NewOrderCore,
    peg_reference: PegReference,
    quantity: u64,
) -> Result<(), CommandError> {
    core.validate()?;

    if quantity == 0 {
        return Err(CommandError::ZeroQuantity);
    }

    if !peg_reference.can_be_taker() && core.time_in_force.is_immediate() {
        return Err(CommandError::PeggedNonTakerImmediateTif);
    }
    Ok(())
}

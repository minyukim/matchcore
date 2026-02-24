use crate::{PegReference, QuantityPolicy, TimeInForce};

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
    /// The new price of the order
    pub new_price: Option<u64>,
    /// The new quantity policy of the order
    pub new_quantity_policy: Option<QuantityPolicy>,
    /// The new time in force of the order
    pub new_time_in_force: Option<TimeInForce>,
}

/// Represents the patch to a pegged order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeggedPatch {
    /// The new peg reference type
    pub new_peg_reference: Option<PegReference>,
    /// The new quantity of the order
    pub new_quantity: Option<u64>,
    /// The new time in force of the order
    pub new_time_in_force: Option<TimeInForce>,
}

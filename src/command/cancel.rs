use crate::{OrderId, OrderKind};

/// Represents a command to cancel an existing order
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CancelCmd {
    /// The ID of the order to cancel
    pub order_id: OrderId,
    /// The type of the order to cancel
    pub order_kind: OrderKind,
}

use serde::{Deserialize, Serialize};

/// Represents a command to cancel an existing order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancelCmd {
    /// The ID of the order to cancel
    pub order_id: u64,
    /// The type of the order to cancel
    pub order_kind: OrderKind,
}

/// Represents the kind of an order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderKind {
    /// Limit order
    Limit,
    /// Pegged order
    Pegged,
}

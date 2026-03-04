use crate::{OrderId, OrderKind};

use serde::{Deserialize, Serialize};

/// Represents a command to cancel an existing order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancelCmd {
    /// The ID of the order to cancel
    pub order_id: OrderId,
    /// The type of the order to cancel
    pub order_kind: OrderKind,
}

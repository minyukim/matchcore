use crate::order::{OrderType, PegReference, QuantityPolicy, Side, TimeInForce};

use serde::{Deserialize, Serialize};

/// Represents a top-level command for all order types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Command<E = ()> {
    /// The common metadata for all command kinds
    pub meta: CommandMeta,
    /// The kind of command
    pub kind: CommandKind<E>,
}

/// Represents the common metadata for all command kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandMeta {
    /// The sequence number of the command
    pub sequence_number: u64,
    /// The timestamp of the command
    pub timestamp: u64,
}

/// Represents the kind of command
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandKind<E = ()> {
    /// A command to submit a new order
    Submit(SubmitCmd<E>),
    /// A command to amend an existing order
    Amend(AmendCmd),
    /// A command to cancel an existing order
    Cancel(CancelCmd),
}

/// Represents a command to submit a new order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmitCmd<E = ()> {
    /// The order to submit
    pub order: NewOrder<E>,
}

/// Represents a command to amend an existing order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AmendCmd {
    /// The ID of the order to amend
    pub order_id: u64,
    /// The changes to the order
    pub changes: AmendChanges,
}

/// Represents a command to cancel an existing order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancelCmd {
    /// The ID of the order to cancel
    pub order_id: u64,
    /// The type of the order to cancel
    pub order_type: OrderType,
}

/// Represents a new order for all order types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NewOrder<E = ()> {
    /// A new market order
    Market(NewMarketOrder<E>),
    /// A new limit order
    Limit(NewLimitOrder<E>),
    /// A new pegged order
    Pegged(NewPeggedOrder<E>),
}

/// Represents the changes to an existing order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmendChanges {
    /// The changes to a limit order
    Limit(LimitAmend),
    /// The changes to a pegged order
    Pegged(PeggedAmend),
}

/// Represents a new market order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewMarketOrder<E = ()> {
    /// The quantity of the order
    pub quantity: u64,
    /// The side of the order
    pub side: Side,
    /// Whether to convert the order to a limit order
    /// if it is not filled immediately at the best available price
    pub market_to_limit: bool,
    /// Additional custom fields
    pub extra: E,
}

/// Represents a new limit order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewLimitOrder<E = ()> {
    /// The core order data
    pub core: NewOrderCore<E>,
    /// The price of the order
    pub price: u64,
    /// The quantity policy of the order
    pub quantity_policy: QuantityPolicy,
}

/// Represents a new pegged order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewPeggedOrder<E = ()> {
    /// The core order data
    pub core: NewOrderCore<E>,
    /// The peg reference type
    pub peg_reference: PegReference,
    /// The quantity of the order
    pub quantity: u64,
}

/// Represents the core data for a new order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewOrderCore<E = ()> {
    /// The side of the order
    pub side: Side,
    /// Whether the order is post-only
    pub post_only: bool,
    /// The time in force of the order
    pub time_in_force: TimeInForce,
    /// Additional custom fields
    pub extra: E,
}

/// Represents the changes to a limit order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LimitAmend {
    /// The new price of the order
    pub new_price: Option<u64>,
    /// The new quantity policy of the order
    pub new_quantity_policy: Option<QuantityPolicy>,
    /// The new time in force of the order
    pub new_time_in_force: Option<TimeInForce>,
}

/// Represents the changes to a pegged order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeggedAmend {
    /// The new peg reference type
    pub new_peg_reference: Option<PegReference>,
    /// The new quantity of the order
    pub new_quantity: Option<u64>,
    /// The new time in force of the order
    pub new_time_in_force: Option<TimeInForce>,
}

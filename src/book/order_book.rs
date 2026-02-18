use crate::{
    Order, PeggedOrder,
    book::{PegLevel, PriceLevel},
};

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

const PEG_REFERENCE_COUNT: usize = 4;

/// Order book that manages orders and levels.
/// It supports adding, updating, cancelling, and matching orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook<E = ()>
where
    E: Clone + Copy + Eq + Serialize + core::fmt::Debug,
{
    /// The symbol for this order book
    symbol: String,

    /// The last price at which a trade occurred, `None` if no trade has occurred yet
    last_trade_price: Option<u64>,

    /// Bid side price levels, stored in a ordered map with O(log N) ordering
    bids: BTreeMap<u64, PriceLevel>,

    /// Ask side price levels, stored in a ordered map with O(log N) ordering
    asks: BTreeMap<u64, PriceLevel>,

    /// Orders indexed by order ID for O(1) lookup
    orders: HashMap<u64, Order<E>>,

    /// Pegged bid side levels, one for each reference price type
    pegged_bids: [PegLevel; PEG_REFERENCE_COUNT],

    /// Pegged ask side levels, one for each reference price type
    pegged_asks: [PegLevel; PEG_REFERENCE_COUNT],

    /// Pegged orders indexed by order ID for O(1) lookup
    pegged_orders: HashMap<u64, PeggedOrder<E>>,
}

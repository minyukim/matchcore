use std::fmt;

use serde::{Deserialize, Serialize};

/// Reference price type for pegged orders
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PegReference {
    /// Pegged to best bid price
    BestBid,
    /// Pegged to best ask price
    BestAsk,
    /// Pegged to mid price between bid and ask
    MidPrice,
    /// Pegged to last trade price
    LastTrade,
}

impl fmt::Display for PegReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PegReference::BestBid => write!(f, "BestBid"),
            PegReference::BestAsk => write!(f, "BestAsk"),
            PegReference::MidPrice => write!(f, "MidPrice"),
            PegReference::LastTrade => write!(f, "LastTrade"),
        }
    }
}

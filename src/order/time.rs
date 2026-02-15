use std::fmt;

use serde::{Deserialize, Serialize};

/// Specifies how long an order remains active before it is executed or expires.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeInForce {
    /// Good 'Til Canceled - The order remains active until it is filled or canceled.
    #[serde(rename = "GTC")]
    Gtc,

    /// Immediate Or Cancel - The order must be filled immediately in its entirety.
    /// If the order cannot be filled completely, the unfilled portion is canceled.
    #[serde(rename = "IOC")]
    Ioc,

    /// Fill Or Kill - The order must be filled immediately and completely.
    /// If the order cannot be filled entirely, the entire order is canceled.
    #[serde(rename = "FOK")]
    Fok,

    /// Good 'Til Date - The order remains active until a specified date and time.
    /// The date and time is expressed as a Unix timestamp (seconds since epoch).
    #[serde(rename = "GTD")]
    Gtd(u64),
}

impl TimeInForce {
    /// Returns true if the order should be canceled after attempting to match
    pub fn is_immediate(&self) -> bool {
        matches!(self, Self::Ioc | Self::Fok)
    }

    /// Returns true if the order has a specific expiration time
    pub fn has_expiry(&self) -> bool {
        matches!(self, Self::Gtd(_))
    }

    /// Checks if an order with this time in force has expired
    pub fn is_expired(&self, timestamp: u64) -> bool {
        match self {
            Self::Gtd(expiry) => timestamp >= *expiry,
            _ => false,
        }
    }
}

impl fmt::Display for TimeInForce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeInForce::Gtc => write!(f, "GTC"),
            TimeInForce::Ioc => write!(f, "IOC"),
            TimeInForce::Fok => write!(f, "FOK"),
            TimeInForce::Gtd(expiry) => write!(f, "GTD-{expiry}"),
        }
    }
}

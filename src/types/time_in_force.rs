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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_immediate() {
        assert!(TimeInForce::Ioc.is_immediate());
        assert!(TimeInForce::Fok.is_immediate());
        assert!(!TimeInForce::Gtc.is_immediate());
        assert!(!TimeInForce::Gtd(1000).is_immediate());
    }

    #[test]
    fn test_has_expiry() {
        assert!(TimeInForce::Gtd(1000).has_expiry());
        assert!(!TimeInForce::Gtc.has_expiry());
        assert!(!TimeInForce::Ioc.has_expiry());
        assert!(!TimeInForce::Fok.has_expiry());
    }

    #[test]
    fn test_is_expired_gtd() {
        let expiry_time = 1000;
        let tif = TimeInForce::Gtd(expiry_time);
        assert!(!tif.is_expired(999));
        assert!(tif.is_expired(1000));
        assert!(tif.is_expired(1001));
    }

    #[test]
    fn test_non_expiring_types() {
        assert!(!TimeInForce::Gtc.is_expired(9999));
        assert!(!TimeInForce::Ioc.is_expired(9999));
        assert!(!TimeInForce::Fok.is_expired(9999));
    }

    #[test]
    fn test_serialize() {
        assert_eq!(serde_json::to_string(&TimeInForce::Gtc).unwrap(), "\"GTC\"");
        assert_eq!(serde_json::to_string(&TimeInForce::Ioc).unwrap(), "\"IOC\"");
        assert_eq!(serde_json::to_string(&TimeInForce::Fok).unwrap(), "\"FOK\"");
        assert_eq!(
            serde_json::to_string(&TimeInForce::Gtd(12345)).unwrap(),
            "{\"GTD\":12345}"
        );
    }

    #[test]
    fn test_deserialize() {
        assert_eq!(
            serde_json::from_str::<TimeInForce>("\"GTC\"").unwrap(),
            TimeInForce::Gtc
        );
        assert_eq!(
            serde_json::from_str::<TimeInForce>("\"IOC\"").unwrap(),
            TimeInForce::Ioc
        );
        assert_eq!(
            serde_json::from_str::<TimeInForce>("\"FOK\"").unwrap(),
            TimeInForce::Fok
        );
        assert_eq!(
            serde_json::from_str::<TimeInForce>("{\"GTD\":12345}").unwrap(),
            TimeInForce::Gtd(12345)
        );
    }

    #[test]
    fn test_round_trip_serialization() {
        for tif in [
            TimeInForce::Gtc,
            TimeInForce::Ioc,
            TimeInForce::Fok,
            TimeInForce::Gtd(12345),
        ] {
            let serialized = serde_json::to_string(&tif).unwrap();
            let deserialized: TimeInForce = serde_json::from_str(&serialized).unwrap();
            assert_eq!(tif, deserialized);
        }
    }

    #[test]
    fn test_invalid_deserialization() {
        assert!(serde_json::from_str::<TimeInForce>("\"Invalid\"").is_err());
        assert!(serde_json::from_str::<TimeInForce>("{\"GTD\":\"not_a_number\"}").is_err());
        assert!(serde_json::from_str::<TimeInForce>("{\"InvalidType\":12345}").is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(TimeInForce::Gtc.to_string(), "GTC");
        assert_eq!(TimeInForce::Ioc.to_string(), "IOC");
        assert_eq!(TimeInForce::Fok.to_string(), "FOK");
        assert_eq!(
            TimeInForce::Gtd(1616823000000).to_string(),
            "GTD-1616823000000"
        );
    }
}

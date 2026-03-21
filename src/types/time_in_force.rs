use super::Timestamp;

use std::fmt;

/// Specifies how long an order remains active before it is executed or expires.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeInForce {
    /// Good 'Til Canceled - The order remains active until it is filled or canceled.
    #[cfg_attr(feature = "serde", serde(rename = "GTC"))]
    Gtc,

    /// Immediate Or Cancel - The order must be filled immediately in its entirety.
    /// If the order cannot be filled completely, the unfilled portion is canceled.
    #[cfg_attr(feature = "serde", serde(rename = "IOC"))]
    Ioc,

    /// Fill Or Kill - The order must be filled immediately and completely.
    /// If the order cannot be filled entirely, the entire order is canceled.
    #[cfg_attr(feature = "serde", serde(rename = "FOK"))]
    Fok,

    /// Good 'Til Date - The order remains active until a specified date and time.
    #[cfg_attr(feature = "serde", serde(rename = "GTD"))]
    Gtd(Timestamp),
}

impl TimeInForce {
    /// Check if the order should be canceled after attempting to match
    pub fn is_immediate(&self) -> bool {
        matches!(self, Self::Ioc | Self::Fok)
    }

    /// Check if the order has an expiry time
    pub fn has_expiry(&self) -> bool {
        matches!(self, Self::Gtd(_))
    }

    /// Get the timestamp when the order expires, if any
    pub fn expires_at(&self) -> Option<Timestamp> {
        match self {
            Self::Gtd(expiry) => Some(*expiry),
            _ => None,
        }
    }

    /// Checks if an order with this time in force has expired
    pub fn is_expired(&self, timestamp: Timestamp) -> bool {
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
        assert!(!TimeInForce::Gtd(Timestamp(1000)).is_immediate());
    }

    #[test]
    fn test_has_expiry() {
        assert!(TimeInForce::Gtd(Timestamp(1000)).has_expiry());
        assert!(!TimeInForce::Gtc.has_expiry());
        assert!(!TimeInForce::Ioc.has_expiry());
        assert!(!TimeInForce::Fok.has_expiry());
    }

    #[test]
    fn test_expired_at() {
        assert_eq!(
            TimeInForce::Gtd(Timestamp(1000)).expires_at(),
            Some(Timestamp(1000))
        );
        assert_eq!(TimeInForce::Gtc.expires_at(), None);
        assert_eq!(TimeInForce::Ioc.expires_at(), None);
        assert_eq!(TimeInForce::Fok.expires_at(), None);
    }

    #[test]
    fn test_is_expired_gtd() {
        let expiry_time = Timestamp(1000);
        let tif = TimeInForce::Gtd(expiry_time);
        assert!(!tif.is_expired(Timestamp(999)));
        assert!(tif.is_expired(Timestamp(1000)));
        assert!(tif.is_expired(Timestamp(1001)));
    }

    #[test]
    fn test_non_expiring_types() {
        assert!(!TimeInForce::Gtc.is_expired(Timestamp(9999)));
        assert!(!TimeInForce::Ioc.is_expired(Timestamp(9999)));
        assert!(!TimeInForce::Fok.is_expired(Timestamp(9999)));
    }

    #[test]
    fn test_display() {
        assert_eq!(TimeInForce::Gtc.to_string(), "GTC");
        assert_eq!(TimeInForce::Ioc.to_string(), "IOC");
        assert_eq!(TimeInForce::Fok.to_string(), "FOK");
        assert_eq!(
            TimeInForce::Gtd(Timestamp(1616823000000)).to_string(),
            "GTD-1616823000000"
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serialize() {
        assert_eq!(serde_json::to_string(&TimeInForce::Gtc).unwrap(), "\"GTC\"");
        assert_eq!(serde_json::to_string(&TimeInForce::Ioc).unwrap(), "\"IOC\"");
        assert_eq!(serde_json::to_string(&TimeInForce::Fok).unwrap(), "\"FOK\"");
        assert_eq!(
            serde_json::to_string(&TimeInForce::Gtd(Timestamp(12345))).unwrap(),
            "{\"GTD\":12345}"
        );
    }

    #[cfg(feature = "serde")]
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
            TimeInForce::Gtd(Timestamp(12345))
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_round_trip_serialization() {
        for tif in [
            TimeInForce::Gtc,
            TimeInForce::Ioc,
            TimeInForce::Fok,
            TimeInForce::Gtd(Timestamp(12345)),
        ] {
            let serialized = serde_json::to_string(&tif).unwrap();
            let deserialized: TimeInForce = serde_json::from_str(&serialized).unwrap();
            assert_eq!(tif, deserialized);
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_invalid_deserialization() {
        assert!(serde_json::from_str::<TimeInForce>("\"Invalid\"").is_err());
        assert!(serde_json::from_str::<TimeInForce>("{\"GTD\":\"not_a_number\"}").is_err());
        assert!(serde_json::from_str::<TimeInForce>("{\"InvalidType\":12345}").is_err());
    }
}

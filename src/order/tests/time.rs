#[cfg(test)]
mod tests_time_in_force {
    use crate::order::TimeInForce;

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
        let test_cases = vec![
            TimeInForce::Gtc,
            TimeInForce::Ioc,
            TimeInForce::Fok,
            TimeInForce::Gtd(12345),
        ];

        for tif in test_cases {
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

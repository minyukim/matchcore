#[cfg(test)]
mod tests_qty_policy {
    use crate::orders::QtyPolicy;

    #[test]
    fn test_visible_qty() {
        assert_eq!(QtyPolicy::Standard { qty: 100 }.visible_qty(), 100);
        assert_eq!(
            QtyPolicy::Iceberg {
                visible: 10,
                hidden: 50,
                replenish: 10
            }
            .visible_qty(),
            10
        );
    }

    #[test]
    fn test_hidden_qty() {
        assert_eq!(QtyPolicy::Standard { qty: 100 }.hidden_qty(), 0);
        assert_eq!(
            QtyPolicy::Iceberg {
                visible: 10,
                hidden: 50,
                replenish: 10
            }
            .hidden_qty(),
            50
        );
    }

    #[test]
    fn test_replenish_amount() {
        assert_eq!(QtyPolicy::Standard { qty: 100 }.replenish_amount(), 0);
        assert_eq!(
            QtyPolicy::Iceberg {
                visible: 10,
                hidden: 50,
                replenish: 10
            }
            .replenish_amount(),
            10
        );
    }

    #[test]
    fn test_serialize() {
        assert_eq!(
            serde_json::to_string(&QtyPolicy::Standard { qty: 100 }).unwrap(),
            "{\"Standard\":{\"qty\":100}}"
        );
        assert_eq!(
            serde_json::to_string(&QtyPolicy::Iceberg {
                visible: 10,
                hidden: 50,
                replenish: 10
            })
            .unwrap(),
            "{\"Iceberg\":{\"visible\":10,\"hidden\":50,\"replenish\":10}}"
        );
    }

    #[test]
    fn test_deserialize() {
        assert_eq!(
            serde_json::from_str::<QtyPolicy>("{\"Standard\":{\"qty\":100}}").unwrap(),
            QtyPolicy::Standard { qty: 100 }
        );
        assert_eq!(
            serde_json::from_str::<QtyPolicy>(
                "{\"Iceberg\":{\"visible\":10,\"hidden\":50,\"replenish\":10}}"
            )
            .unwrap(),
            QtyPolicy::Iceberg {
                visible: 10,
                hidden: 50,
                replenish: 10
            }
        );
    }

    #[test]
    fn test_round_trip_serialization() {
        let test_cases = vec![
            QtyPolicy::Standard { qty: 100 },
            QtyPolicy::Iceberg {
                visible: 10,
                hidden: 50,
                replenish: 10,
            },
        ];

        for qty_policy in test_cases {
            let serialized = serde_json::to_string(&qty_policy).unwrap();
            let deserialized: QtyPolicy = serde_json::from_str(&serialized).unwrap();
            assert_eq!(qty_policy, deserialized);
        }
    }

    #[test]
    fn test_invalid_deserialization() {
        assert!(serde_json::from_str::<QtyPolicy>("\"Invalid\"").is_err());
        assert!(
            serde_json::from_str::<QtyPolicy>("{\"Standard\":{\"qty\":\"not_a_number\"}}").is_err()
        );
        assert!(serde_json::from_str::<QtyPolicy>("{\"Iceberg\":{\"visible\":\"not_a_number\",\"hidden\":\"not_a_number\",\"replenish\":\"not_a_number\"}}").is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(
            QtyPolicy::Standard { qty: 100 }.to_string(),
            "Standard: 100"
        );
        assert_eq!(
            QtyPolicy::Iceberg {
                visible: 10,
                hidden: 50,
                replenish: 10
            }
            .to_string(),
            "Iceberg: visible=10, hidden=50, replenish=10"
        );
    }
}

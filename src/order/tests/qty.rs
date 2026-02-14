#[cfg(test)]
mod tests_qty_policy {
    use crate::order::QtyPolicy;

    #[test]
    fn test_visible_qty() {
        assert_eq!(QtyPolicy::Standard { qty: 100 }.visible_qty(), 100);
        assert_eq!(
            QtyPolicy::Iceberg {
                visible_qty: 10,
                hidden_qty: 50,
                replenish_size: 10
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
                visible_qty: 10,
                hidden_qty: 50,
                replenish_size: 10
            }
            .hidden_qty(),
            50
        );
    }

    #[test]
    fn test_replenish_size() {
        assert_eq!(QtyPolicy::Standard { qty: 100 }.replenish_size(), 0);
        assert_eq!(
            QtyPolicy::Iceberg {
                visible_qty: 10,
                hidden_qty: 50,
                replenish_size: 10
            }
            .replenish_size(),
            10
        );
    }

    #[test]
    fn test_update_visible_qty() {
        {
            let mut standard_qty = QtyPolicy::Standard { qty: 100 };
            standard_qty.update_visible_qty(50);

            assert_eq!(standard_qty.visible_qty(), 50);
            assert_eq!(standard_qty.hidden_qty(), 0);
            assert_eq!(standard_qty.replenish_size(), 0);
        }
        {
            let mut iceberg_qty = QtyPolicy::Iceberg {
                visible_qty: 10,
                hidden_qty: 50,
                replenish_size: 10,
            };
            iceberg_qty.update_visible_qty(20);

            assert_eq!(iceberg_qty.visible_qty(), 20);
            assert_eq!(iceberg_qty.hidden_qty(), 50);
            assert_eq!(iceberg_qty.replenish_size(), 10);
        }
    }

    #[test]
    fn test_replenish() {
        {
            let mut standard_qty = QtyPolicy::Standard { qty: 100 };

            assert_eq!(standard_qty.replenish(), 0);
            assert_eq!(standard_qty.visible_qty(), 100);
            assert_eq!(standard_qty.hidden_qty(), 0);
            assert_eq!(standard_qty.replenish_size(), 0);
        }
        {
            let mut iceberg_qty = QtyPolicy::Iceberg {
                visible_qty: 0,
                hidden_qty: 25,
                replenish_size: 10,
            };

            assert_eq!(iceberg_qty.replenish(), 10);
            assert_eq!(iceberg_qty.visible_qty(), 10);
            assert_eq!(iceberg_qty.hidden_qty(), 15);
            assert_eq!(iceberg_qty.replenish_size(), 10);

            assert_eq!(iceberg_qty.replenish(), 10);
            assert_eq!(iceberg_qty.visible_qty(), 20);
            assert_eq!(iceberg_qty.hidden_qty(), 5);
            assert_eq!(iceberg_qty.replenish_size(), 10);

            assert_eq!(iceberg_qty.replenish(), 5);
            assert_eq!(iceberg_qty.visible_qty(), 25);
            assert_eq!(iceberg_qty.hidden_qty(), 0);
            assert_eq!(iceberg_qty.replenish_size(), 10);

            assert_eq!(iceberg_qty.replenish(), 0);
            assert_eq!(iceberg_qty.visible_qty(), 25);
            assert_eq!(iceberg_qty.hidden_qty(), 0);
            assert_eq!(iceberg_qty.replenish_size(), 10);
        }
    }

    #[test]
    fn test_serialize() {
        assert_eq!(
            serde_json::to_string(&QtyPolicy::Standard { qty: 100 }).unwrap(),
            "{\"Standard\":{\"qty\":100}}"
        );
        assert_eq!(
            serde_json::to_string(&QtyPolicy::Iceberg {
                visible_qty: 10,
                hidden_qty: 50,
                replenish_size: 10
            })
            .unwrap(),
            "{\"Iceberg\":{\"visible_qty\":10,\"hidden_qty\":50,\"replenish_size\":10}}"
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
                "{\"Iceberg\":{\"visible_qty\":10,\"hidden_qty\":50,\"replenish_size\":10}}"
            )
            .unwrap(),
            QtyPolicy::Iceberg {
                visible_qty: 10,
                hidden_qty: 50,
                replenish_size: 10
            }
        );
    }

    #[test]
    fn test_round_trip_serialization() {
        let test_cases = vec![
            QtyPolicy::Standard { qty: 100 },
            QtyPolicy::Iceberg {
                visible_qty: 10,
                hidden_qty: 50,
                replenish_size: 10,
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
        assert!(serde_json::from_str::<QtyPolicy>("{\"Iceberg\":{\"visible_qty\":\"not_a_number\",\"hidden_qty\":\"not_a_number\",\"replenish_size\":\"not_a_number\"}}").is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(
            QtyPolicy::Standard { qty: 100 }.to_string(),
            "Standard: 100"
        );
        assert_eq!(
            QtyPolicy::Iceberg {
                visible_qty: 10,
                hidden_qty: 50,
                replenish_size: 10
            }
            .to_string(),
            "Iceberg: visible_qty=10 hidden_qty=50 replenish_size=10"
        );
    }
}

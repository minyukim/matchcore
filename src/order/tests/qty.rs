#[cfg(test)]
mod tests_quantity_policy {
    use crate::order::QuantityPolicy;

    #[test]
    fn test_visible_qty() {
        assert_eq!(QuantityPolicy::Standard { qty: 100 }.visible_qty(), 100);
        assert_eq!(
            QuantityPolicy::Iceberg {
                visible_qty: 10,
                hidden_qty: 50,
                replenish_qty: 10
            }
            .visible_qty(),
            10
        );
    }

    #[test]
    fn test_hidden_qty() {
        assert_eq!(QuantityPolicy::Standard { qty: 100 }.hidden_qty(), 0);
        assert_eq!(
            QuantityPolicy::Iceberg {
                visible_qty: 10,
                hidden_qty: 50,
                replenish_qty: 10
            }
            .hidden_qty(),
            50
        );
    }

    #[test]
    fn test_replenish_qty() {
        assert_eq!(QuantityPolicy::Standard { qty: 100 }.replenish_qty(), 0);
        assert_eq!(
            QuantityPolicy::Iceberg {
                visible_qty: 10,
                hidden_qty: 50,
                replenish_qty: 10
            }
            .replenish_qty(),
            10
        );
    }

    #[test]
    fn test_update_visible_qty() {
        {
            let mut standard_qty = QuantityPolicy::Standard { qty: 100 };
            standard_qty.update_visible_qty(50);

            assert_eq!(standard_qty.visible_qty(), 50);
            assert_eq!(standard_qty.hidden_qty(), 0);
            assert_eq!(standard_qty.replenish_qty(), 0);
        }
        {
            let mut iceberg_qty = QuantityPolicy::Iceberg {
                visible_qty: 10,
                hidden_qty: 50,
                replenish_qty: 10,
            };
            iceberg_qty.update_visible_qty(20);

            assert_eq!(iceberg_qty.visible_qty(), 20);
            assert_eq!(iceberg_qty.hidden_qty(), 50);
            assert_eq!(iceberg_qty.replenish_qty(), 10);
        }
    }

    #[test]
    fn test_replenish() {
        {
            let mut standard_qty = QuantityPolicy::Standard { qty: 100 };

            assert_eq!(standard_qty.replenish(), 0);
            assert_eq!(standard_qty.visible_qty(), 100);
            assert_eq!(standard_qty.hidden_qty(), 0);
            assert_eq!(standard_qty.replenish_qty(), 0);
        }
        {
            let mut iceberg_qty = QuantityPolicy::Iceberg {
                visible_qty: 0,
                hidden_qty: 25,
                replenish_qty: 10,
            };

            assert_eq!(iceberg_qty.replenish(), 10);
            assert_eq!(iceberg_qty.visible_qty(), 10);
            assert_eq!(iceberg_qty.hidden_qty(), 15);
            assert_eq!(iceberg_qty.replenish_qty(), 10);

            assert_eq!(iceberg_qty.replenish(), 10);
            assert_eq!(iceberg_qty.visible_qty(), 20);
            assert_eq!(iceberg_qty.hidden_qty(), 5);
            assert_eq!(iceberg_qty.replenish_qty(), 10);

            assert_eq!(iceberg_qty.replenish(), 5);
            assert_eq!(iceberg_qty.visible_qty(), 25);
            assert_eq!(iceberg_qty.hidden_qty(), 0);
            assert_eq!(iceberg_qty.replenish_qty(), 10);

            assert_eq!(iceberg_qty.replenish(), 0);
            assert_eq!(iceberg_qty.visible_qty(), 25);
            assert_eq!(iceberg_qty.hidden_qty(), 0);
            assert_eq!(iceberg_qty.replenish_qty(), 10);
        }
    }

    #[test]
    fn test_serialize() {
        assert_eq!(
            serde_json::to_string(&QuantityPolicy::Standard { qty: 100 }).unwrap(),
            "{\"Standard\":{\"qty\":100}}"
        );
        assert_eq!(
            serde_json::to_string(&QuantityPolicy::Iceberg {
                visible_qty: 10,
                hidden_qty: 50,
                replenish_qty: 10
            })
            .unwrap(),
            "{\"Iceberg\":{\"visible_qty\":10,\"hidden_qty\":50,\"replenish_qty\":10}}"
        );
    }

    #[test]
    fn test_deserialize() {
        assert_eq!(
            serde_json::from_str::<QuantityPolicy>("{\"Standard\":{\"qty\":100}}").unwrap(),
            QuantityPolicy::Standard { qty: 100 }
        );
        assert_eq!(
            serde_json::from_str::<QuantityPolicy>(
                "{\"Iceberg\":{\"visible_qty\":10,\"hidden_qty\":50,\"replenish_qty\":10}}"
            )
            .unwrap(),
            QuantityPolicy::Iceberg {
                visible_qty: 10,
                hidden_qty: 50,
                replenish_qty: 10
            }
        );
    }

    #[test]
    fn test_round_trip_serialization() {
        for qty_policy in [
            QuantityPolicy::Standard { qty: 100 },
            QuantityPolicy::Iceberg {
                visible_qty: 10,
                hidden_qty: 50,
                replenish_qty: 10,
            },
        ] {
            let serialized = serde_json::to_string(&qty_policy).unwrap();
            let deserialized: QuantityPolicy = serde_json::from_str(&serialized).unwrap();
            assert_eq!(qty_policy, deserialized);
        }
    }

    #[test]
    fn test_invalid_deserialization() {
        assert!(serde_json::from_str::<QuantityPolicy>("\"Invalid\"").is_err());
        assert!(
            serde_json::from_str::<QuantityPolicy>("{\"Standard\":{\"qty\":\"not_a_number\"}}")
                .is_err()
        );
        assert!(serde_json::from_str::<QuantityPolicy>("{\"Iceberg\":{\"visible_qty\":\"not_a_number\",\"hidden_qty\":\"not_a_number\",\"replenish_qty\":\"not_a_number\"}}").is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(
            QuantityPolicy::Standard { qty: 100 }.to_string(),
            "Standard: 100"
        );
        assert_eq!(
            QuantityPolicy::Iceberg {
                visible_qty: 10,
                hidden_qty: 50,
                replenish_qty: 10
            }
            .to_string(),
            "Iceberg: visible_qty=10 hidden_qty=50 replenish_qty=10"
        );
    }
}

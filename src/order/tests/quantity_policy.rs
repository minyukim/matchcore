#[cfg(test)]
mod tests_quantity_policy {
    use crate::order::QuantityPolicy;

    #[test]
    fn test_visible_quantity() {
        assert_eq!(
            QuantityPolicy::Standard { quantity: 100 }.visible_quantity(),
            100
        );
        assert_eq!(
            QuantityPolicy::Iceberg {
                visible_quantity: 10,
                hidden_quantity: 50,
                replenish_quantity: 10
            }
            .visible_quantity(),
            10
        );
    }

    #[test]
    fn test_hidden_quantity() {
        assert_eq!(
            QuantityPolicy::Standard { quantity: 100 }.hidden_quantity(),
            0
        );
        assert_eq!(
            QuantityPolicy::Iceberg {
                visible_quantity: 10,
                hidden_quantity: 50,
                replenish_quantity: 10
            }
            .hidden_quantity(),
            50
        );
    }

    #[test]
    fn test_replenish_quantity() {
        assert_eq!(
            QuantityPolicy::Standard { quantity: 100 }.replenish_quantity(),
            0
        );
        assert_eq!(
            QuantityPolicy::Iceberg {
                visible_quantity: 10,
                hidden_quantity: 50,
                replenish_quantity: 10
            }
            .replenish_quantity(),
            10
        );
    }

    #[test]
    fn test_total_quantity() {
        assert_eq!(
            QuantityPolicy::Standard { quantity: 100 }.total_quantity(),
            100
        );
        assert_eq!(
            QuantityPolicy::Iceberg {
                visible_quantity: 10,
                hidden_quantity: 50,
                replenish_quantity: 10
            }
            .total_quantity(),
            60
        );
    }

    #[test]
    fn test_is_filled() {
        assert!(!QuantityPolicy::Standard { quantity: 100 }.is_filled());
        assert!(QuantityPolicy::Standard { quantity: 0 }.is_filled());

        assert!(
            !QuantityPolicy::Iceberg {
                visible_quantity: 10,
                hidden_quantity: 50,
                replenish_quantity: 10
            }
            .is_filled()
        );
        assert!(
            QuantityPolicy::Iceberg {
                visible_quantity: 0,
                hidden_quantity: 0,
                replenish_quantity: 10
            }
            .is_filled()
        );
    }

    #[test]
    fn test_update_visible_quantity() {
        {
            let mut standard_quantity = QuantityPolicy::Standard { quantity: 100 };
            standard_quantity.update_visible_quantity(50);

            assert_eq!(standard_quantity.visible_quantity(), 50);
        }
        {
            let mut iceberg_quantity = QuantityPolicy::Iceberg {
                visible_quantity: 10,
                hidden_quantity: 50,
                replenish_quantity: 10,
            };
            iceberg_quantity.update_visible_quantity(20);

            assert_eq!(iceberg_quantity.visible_quantity(), 20);
        }
    }

    #[test]
    fn test_replenish() {
        {
            let mut standard_quantity = QuantityPolicy::Standard { quantity: 100 };

            assert_eq!(standard_quantity.replenish(), 0);
            assert_eq!(standard_quantity.visible_quantity(), 100);
            assert_eq!(standard_quantity.hidden_quantity(), 0);
            assert_eq!(standard_quantity.replenish_quantity(), 0);
        }
        {
            let mut iceberg_quantity = QuantityPolicy::Iceberg {
                visible_quantity: 0,
                hidden_quantity: 25,
                replenish_quantity: 10,
            };

            assert_eq!(iceberg_quantity.replenish(), 10);
            assert_eq!(iceberg_quantity.visible_quantity(), 10);
            assert_eq!(iceberg_quantity.hidden_quantity(), 15);
            assert_eq!(iceberg_quantity.replenish_quantity(), 10);

            assert_eq!(iceberg_quantity.replenish(), 10);
            assert_eq!(iceberg_quantity.visible_quantity(), 20);
            assert_eq!(iceberg_quantity.hidden_quantity(), 5);
            assert_eq!(iceberg_quantity.replenish_quantity(), 10);

            assert_eq!(iceberg_quantity.replenish(), 5);
            assert_eq!(iceberg_quantity.visible_quantity(), 25);
            assert_eq!(iceberg_quantity.hidden_quantity(), 0);
            assert_eq!(iceberg_quantity.replenish_quantity(), 10);

            assert_eq!(iceberg_quantity.replenish(), 0);
            assert_eq!(iceberg_quantity.visible_quantity(), 25);
            assert_eq!(iceberg_quantity.hidden_quantity(), 0);
            assert_eq!(iceberg_quantity.replenish_quantity(), 10);
        }
    }

    #[test]
    fn test_serialize() {
        assert_eq!(
            serde_json::to_string(&QuantityPolicy::Standard { quantity: 100 }).unwrap(),
            "{\"Standard\":{\"quantity\":100}}"
        );
        assert_eq!(
            serde_json::to_string(&QuantityPolicy::Iceberg {
                visible_quantity: 10,
                hidden_quantity: 50,
                replenish_quantity: 10
            })
            .unwrap(),
            "{\"Iceberg\":{\"visible_quantity\":10,\"hidden_quantity\":50,\"replenish_quantity\":10}}"
        );
    }

    #[test]
    fn test_deserialize() {
        assert_eq!(
            serde_json::from_str::<QuantityPolicy>("{\"Standard\":{\"quantity\":100}}").unwrap(),
            QuantityPolicy::Standard { quantity: 100 }
        );
        assert_eq!(
            serde_json::from_str::<QuantityPolicy>(
                "{\"Iceberg\":{\"visible_quantity\":10,\"hidden_quantity\":50,\"replenish_quantity\":10}}"
            )
            .unwrap(),
            QuantityPolicy::Iceberg {
                visible_quantity: 10,
                hidden_quantity: 50,
                replenish_quantity: 10
            }
        );
    }

    #[test]
    fn test_round_trip_serialization() {
        for quantity_policy in [
            QuantityPolicy::Standard { quantity: 100 },
            QuantityPolicy::Iceberg {
                visible_quantity: 10,
                hidden_quantity: 50,
                replenish_quantity: 10,
            },
        ] {
            let serialized = serde_json::to_string(&quantity_policy).unwrap();
            let deserialized: QuantityPolicy = serde_json::from_str(&serialized).unwrap();
            assert_eq!(quantity_policy, deserialized);
        }
    }

    #[test]
    fn test_invalid_deserialization() {
        assert!(serde_json::from_str::<QuantityPolicy>("\"Invalid\"").is_err());
        assert!(
            serde_json::from_str::<QuantityPolicy>(
                "{\"Standard\":{\"quantity\":\"not_a_number\"}}"
            )
            .is_err()
        );
        assert!(serde_json::from_str::<QuantityPolicy>("{\"Iceberg\":{\"visible_quantity\":\"not_a_number\",\"hidden_quantity\":\"not_a_number\",\"replenish_quantity\":\"not_a_number\"}}").is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(
            QuantityPolicy::Standard { quantity: 100 }.to_string(),
            "Standard: 100"
        );
        assert_eq!(
            QuantityPolicy::Iceberg {
                visible_quantity: 10,
                hidden_quantity: 50,
                replenish_quantity: 10
            }
            .to_string(),
            "Iceberg: visible_quantity=10 hidden_quantity=50 replenish_quantity=10"
        );
    }
}

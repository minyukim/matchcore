use super::Quantity;

use std::fmt;

/// Represents the quantity policy of an order
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantityPolicy {
    /// Standard quantity policy
    Standard {
        /// The quantity of the order
        quantity: Quantity,
    },
    /// Iceberg quantity policy
    Iceberg {
        /// The visible quantity of the order
        visible_quantity: Quantity,
        /// The hidden quantity of the order
        hidden_quantity: Quantity,
        /// The replenish quantity of the order
        replenish_quantity: Quantity,
    },
}

impl QuantityPolicy {
    /// Get the quantity of the order
    pub fn visible_quantity(&self) -> Quantity {
        match self {
            QuantityPolicy::Standard { quantity } => *quantity,
            QuantityPolicy::Iceberg {
                visible_quantity, ..
            } => *visible_quantity,
        }
    }

    /// Get the hidden quantity of the order
    pub fn hidden_quantity(&self) -> Quantity {
        match self {
            QuantityPolicy::Iceberg {
                hidden_quantity, ..
            } => *hidden_quantity,
            _ => Quantity(0),
        }
    }

    /// Get the replenish quantity of the order
    pub fn replenish_quantity(&self) -> Quantity {
        match self {
            QuantityPolicy::Iceberg {
                replenish_quantity, ..
            } => *replenish_quantity,
            _ => Quantity(0),
        }
    }

    /// Get the total quantity of the order
    pub fn total_quantity(&self) -> Quantity {
        self.visible_quantity() + self.hidden_quantity()
    }

    /// Check if the order is filled
    pub fn is_filled(&self) -> bool {
        self.total_quantity().is_zero()
    }

    /// Update the visible quantity of the order
    pub fn update_visible_quantity(&mut self, new_visible_quantity: Quantity) {
        match self {
            QuantityPolicy::Standard { quantity } => *quantity = new_visible_quantity,
            QuantityPolicy::Iceberg {
                visible_quantity, ..
            } => *visible_quantity = new_visible_quantity,
        }
    }

    /// Replenish the hidden quantity of the order
    /// Returns the quantity replenished
    pub fn replenish(&mut self) -> Quantity {
        match self {
            QuantityPolicy::Iceberg {
                visible_quantity,
                hidden_quantity,
                replenish_quantity,
            } => {
                let new_hidden = hidden_quantity.saturating_sub(*replenish_quantity);
                let replenished = *hidden_quantity - new_hidden;

                *visible_quantity = visible_quantity.saturating_add(replenished);
                *hidden_quantity = new_hidden;

                replenished
            }
            _ => Quantity(0),
        }
    }
}

impl fmt::Display for QuantityPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuantityPolicy::Standard { quantity } => write!(f, "Standard: {}", quantity),
            QuantityPolicy::Iceberg {
                visible_quantity,
                hidden_quantity,
                replenish_quantity,
            } => write!(
                f,
                "Iceberg: visible_quantity={} hidden_quantity={} replenish_quantity={}",
                visible_quantity, hidden_quantity, replenish_quantity
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visible_quantity() {
        assert_eq!(
            QuantityPolicy::Standard {
                quantity: Quantity(100)
            }
            .visible_quantity(),
            Quantity(100)
        );
        assert_eq!(
            QuantityPolicy::Iceberg {
                visible_quantity: Quantity(10),
                hidden_quantity: Quantity(50),
                replenish_quantity: Quantity(10)
            }
            .visible_quantity(),
            Quantity(10)
        );
    }

    #[test]
    fn test_hidden_quantity() {
        assert_eq!(
            QuantityPolicy::Standard {
                quantity: Quantity(100)
            }
            .hidden_quantity(),
            Quantity(0)
        );
        assert_eq!(
            QuantityPolicy::Iceberg {
                visible_quantity: Quantity(10),
                hidden_quantity: Quantity(50),
                replenish_quantity: Quantity(10)
            }
            .hidden_quantity(),
            Quantity(50)
        );
    }

    #[test]
    fn test_replenish_quantity() {
        assert_eq!(
            QuantityPolicy::Standard {
                quantity: Quantity(100)
            }
            .replenish_quantity(),
            Quantity(0)
        );
        assert_eq!(
            QuantityPolicy::Iceberg {
                visible_quantity: Quantity(10),
                hidden_quantity: Quantity(50),
                replenish_quantity: Quantity(10)
            }
            .replenish_quantity(),
            Quantity(10)
        );
    }

    #[test]
    fn test_total_quantity() {
        assert_eq!(
            QuantityPolicy::Standard {
                quantity: Quantity(100)
            }
            .total_quantity(),
            Quantity(100)
        );
        assert_eq!(
            QuantityPolicy::Iceberg {
                visible_quantity: Quantity(10),
                hidden_quantity: Quantity(50),
                replenish_quantity: Quantity(10)
            }
            .total_quantity(),
            Quantity(60)
        );
    }

    #[test]
    fn test_is_filled() {
        assert!(
            !QuantityPolicy::Standard {
                quantity: Quantity(100)
            }
            .is_filled()
        );
        assert!(
            QuantityPolicy::Standard {
                quantity: Quantity(0)
            }
            .is_filled()
        );

        assert!(
            !QuantityPolicy::Iceberg {
                visible_quantity: Quantity(10),
                hidden_quantity: Quantity(50),
                replenish_quantity: Quantity(10)
            }
            .is_filled()
        );
        assert!(
            QuantityPolicy::Iceberg {
                visible_quantity: Quantity(0),
                hidden_quantity: Quantity(0),
                replenish_quantity: Quantity(10)
            }
            .is_filled()
        );
    }

    #[test]
    fn test_update_visible_quantity() {
        {
            let mut standard_quantity = QuantityPolicy::Standard {
                quantity: Quantity(100),
            };
            standard_quantity.update_visible_quantity(Quantity(50));

            assert_eq!(standard_quantity.visible_quantity(), Quantity(50));
        }
        {
            let mut iceberg_quantity = QuantityPolicy::Iceberg {
                visible_quantity: Quantity(10),
                hidden_quantity: Quantity(50),
                replenish_quantity: Quantity(10),
            };
            iceberg_quantity.update_visible_quantity(Quantity(20));

            assert_eq!(iceberg_quantity.visible_quantity(), Quantity(20));
        }
    }

    #[test]
    fn test_replenish() {
        {
            let mut standard_quantity = QuantityPolicy::Standard {
                quantity: Quantity(100),
            };

            assert_eq!(standard_quantity.replenish(), Quantity(0));
            assert_eq!(standard_quantity.visible_quantity(), Quantity(100));
            assert_eq!(standard_quantity.hidden_quantity(), Quantity(0));
            assert_eq!(standard_quantity.replenish_quantity(), Quantity(0));
        }
        {
            let mut iceberg_quantity = QuantityPolicy::Iceberg {
                visible_quantity: Quantity(0),
                hidden_quantity: Quantity(25),
                replenish_quantity: Quantity(10),
            };

            assert_eq!(iceberg_quantity.replenish(), Quantity(10));
            assert_eq!(iceberg_quantity.visible_quantity(), Quantity(10));
            assert_eq!(iceberg_quantity.hidden_quantity(), Quantity(15));
            assert_eq!(iceberg_quantity.replenish_quantity(), Quantity(10));

            assert_eq!(iceberg_quantity.replenish(), Quantity(10));
            assert_eq!(iceberg_quantity.visible_quantity(), Quantity(20));
            assert_eq!(iceberg_quantity.hidden_quantity(), Quantity(5));
            assert_eq!(iceberg_quantity.replenish_quantity(), Quantity(10));

            assert_eq!(iceberg_quantity.replenish(), Quantity(5));
            assert_eq!(iceberg_quantity.visible_quantity(), Quantity(25));
            assert_eq!(iceberg_quantity.hidden_quantity(), Quantity(0));
            assert_eq!(iceberg_quantity.replenish_quantity(), Quantity(10));

            assert_eq!(iceberg_quantity.replenish(), Quantity(0));
            assert_eq!(iceberg_quantity.visible_quantity(), Quantity(25));
            assert_eq!(iceberg_quantity.hidden_quantity(), Quantity(0));
            assert_eq!(iceberg_quantity.replenish_quantity(), Quantity(10));
        }
    }

    #[test]
    fn test_display() {
        assert_eq!(
            QuantityPolicy::Standard {
                quantity: Quantity(100)
            }
            .to_string(),
            "Standard: 100"
        );
        assert_eq!(
            QuantityPolicy::Iceberg {
                visible_quantity: Quantity(10),
                hidden_quantity: Quantity(50),
                replenish_quantity: Quantity(10)
            }
            .to_string(),
            "Iceberg: visible_quantity=10 hidden_quantity=50 replenish_quantity=10"
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serialize() {
        assert_eq!(
            serde_json::to_string(&QuantityPolicy::Standard {
                quantity: Quantity(100)
            })
            .unwrap(),
            "{\"Standard\":{\"quantity\":100}}"
        );
        assert_eq!(
            serde_json::to_string(&QuantityPolicy::Iceberg {
                visible_quantity: Quantity(10),
                hidden_quantity: Quantity(50),
                replenish_quantity: Quantity(10)
            })
            .unwrap(),
            "{\"Iceberg\":{\"visible_quantity\":10,\"hidden_quantity\":50,\"replenish_quantity\":10}}"
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_deserialize() {
        assert_eq!(
            serde_json::from_str::<QuantityPolicy>("{\"Standard\":{\"quantity\":100}}").unwrap(),
            QuantityPolicy::Standard {
                quantity: Quantity(100)
            }
        );
        assert_eq!(
            serde_json::from_str::<QuantityPolicy>(
                "{\"Iceberg\":{\"visible_quantity\":10,\"hidden_quantity\":50,\"replenish_quantity\":10}}"
            )
            .unwrap(),
            QuantityPolicy::Iceberg {
                visible_quantity: Quantity(10),
                hidden_quantity: Quantity(50),
                replenish_quantity: Quantity(10)
            }
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_round_trip_serialization() {
        for quantity_policy in [
            QuantityPolicy::Standard {
                quantity: Quantity(100),
            },
            QuantityPolicy::Iceberg {
                visible_quantity: Quantity(10),
                hidden_quantity: Quantity(50),
                replenish_quantity: Quantity(10),
            },
        ] {
            let serialized = serde_json::to_string(&quantity_policy).unwrap();
            let deserialized: QuantityPolicy = serde_json::from_str(&serialized).unwrap();
            assert_eq!(quantity_policy, deserialized);
        }
    }

    #[cfg(feature = "serde")]
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
}

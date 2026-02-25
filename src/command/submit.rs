use crate::{PegReference, QuantityPolicy, Side, TimeInForce, command::CommandError};

use serde::{Deserialize, Serialize};

/// Represents a command to submit a new order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmitCmd<E = ()> {
    /// The order to submit
    pub order: NewOrder<E>,
}

/// Represents a new order for all order types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NewOrder<E = ()> {
    /// A new market order
    Market(NewMarketOrder<E>),
    /// A new limit order
    Limit(NewLimitOrder<E>),
    /// A new pegged order
    Pegged(NewPeggedOrder<E>),
}

/// Represents a new market order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewMarketOrder<E = ()> {
    /// The quantity of the order
    pub quantity: u64,
    /// The side of the order
    pub side: Side,
    /// Whether to convert the order to a limit order
    /// if it is not filled immediately at the best available price
    pub market_to_limit: bool,
    /// Additional custom fields
    pub extra: E,
}

impl<E: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug>
    NewMarketOrder<E>
{
    /// Validate the order
    pub fn validate(&self) -> Result<(), CommandError> {
        if self.quantity == 0 {
            return Err(CommandError::ZeroQuantity);
        }
        Ok(())
    }
}

/// Represents a new limit order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewLimitOrder<E = ()> {
    /// The core order data
    pub core: NewOrderCore<E>,
    /// The price of the order
    pub price: u64,
    /// The quantity policy of the order
    pub quantity_policy: QuantityPolicy,
}

impl<E: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug>
    NewLimitOrder<E>
{
    /// Validate the order
    pub fn validate(&self) -> Result<(), CommandError> {
        validate_limit_order_invariants(&self.core, self.price, self.quantity_policy)
    }
}

/// Validate the invariants of a limit order
pub(super) fn validate_limit_order_invariants<
    E: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug,
>(
    core: &NewOrderCore<E>,
    price: u64,
    quantity_policy: QuantityPolicy,
) -> Result<(), CommandError> {
    core.validate()?;

    if price == 0 {
        return Err(CommandError::ZeroPrice);
    }

    match quantity_policy {
        QuantityPolicy::Standard { quantity } => {
            if quantity == 0 {
                return Err(CommandError::ZeroQuantity);
            }
        }
        QuantityPolicy::Iceberg {
            visible_quantity,
            hidden_quantity,
            replenish_quantity,
        } => {
            if visible_quantity == 0 {
                return Err(CommandError::ZeroQuantity);
            }
            if hidden_quantity == 0 {
                return Err(CommandError::IcebergZeroHiddenQuantity);
            }
            if replenish_quantity == 0 {
                return Err(CommandError::IcebergZeroReplenishQuantity);
            }
        }
    }
    Ok(())
}

/// Represents a new pegged order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewPeggedOrder<E = ()> {
    /// The core order data
    pub core: NewOrderCore<E>,
    /// The peg reference type
    pub peg_reference: PegReference,
    /// The quantity of the order
    pub quantity: u64,
}

impl<E: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug>
    NewPeggedOrder<E>
{
    /// Validate the order
    pub fn validate(&self) -> Result<(), CommandError> {
        self.core.validate()?;

        if self.quantity == 0 {
            return Err(CommandError::ZeroQuantity);
        }

        if !self.peg_reference.can_be_taker() && self.core.time_in_force.is_immediate() {
            return Err(CommandError::PeggedNonTakerImmediateTif);
        }
        Ok(())
    }
}

/// Represents the shared core data for all order types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewOrderCore<E = ()> {
    /// The side of the order
    pub side: Side,
    /// Whether the order is post-only
    pub post_only: bool,
    /// The time in force of the order
    pub time_in_force: TimeInForce,
    /// Additional custom fields
    pub extra: E,
}

impl<E: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug>
    NewOrderCore<E>
{
    /// Validate the order core
    pub fn validate(&self) -> Result<(), CommandError> {
        if self.time_in_force.is_immediate() && self.post_only {
            return Err(CommandError::PostOnlyImmediateTif);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_new_market_order() {
        struct Case {
            name: &'static str,
            quantity: u64,
            side: Side,
            market_to_limit: bool,
            expected: Result<(), CommandError>,
        }

        let cases = [
            Case {
                name: "valid market order",
                quantity: 100,
                side: Side::Buy,
                market_to_limit: true,
                expected: Ok(()),
            },
            Case {
                name: "zero quantity",
                quantity: 0,
                side: Side::Buy,
                market_to_limit: true,
                expected: Err(CommandError::ZeroQuantity),
            },
        ];

        for case in cases {
            let order = NewMarketOrder {
                quantity: case.quantity,
                side: case.side,
                market_to_limit: case.market_to_limit,
                extra: (),
            };

            match case.expected {
                Ok(()) => assert!(order.validate().is_ok(), "case: {}", case.name),
                Err(e) => assert_eq!(order.validate().unwrap_err(), e, "case: {}", case.name),
            }
        }
    }

    #[test]
    fn test_validate_new_limit_order() {
        struct Case {
            name: &'static str,
            order: NewLimitOrder<()>,
            expected: Result<(), CommandError>,
        }

        let cases = [
            Case {
                name: "valid standard order",
                order: NewLimitOrder {
                    core: NewOrderCore {
                        side: Side::Buy,
                        post_only: false,
                        time_in_force: TimeInForce::Gtc,
                        extra: (),
                    },
                    price: 100,
                    quantity_policy: QuantityPolicy::Standard { quantity: 10 },
                },
                expected: Ok(()),
            },
            Case {
                name: "zero price",
                order: NewLimitOrder {
                    core: NewOrderCore {
                        side: Side::Buy,
                        post_only: false,
                        time_in_force: TimeInForce::Gtc,
                        extra: (),
                    },
                    price: 0,
                    quantity_policy: QuantityPolicy::Standard { quantity: 10 },
                },
                expected: Err(CommandError::ZeroPrice),
            },
            Case {
                name: "standard zero quantity",
                order: NewLimitOrder {
                    core: NewOrderCore {
                        side: Side::Buy,
                        post_only: false,
                        time_in_force: TimeInForce::Gtc,
                        extra: (),
                    },
                    price: 100,
                    quantity_policy: QuantityPolicy::Standard { quantity: 0 },
                },
                expected: Err(CommandError::ZeroQuantity),
            },
            Case {
                name: "iceberg zero visible quantity",
                order: NewLimitOrder {
                    core: NewOrderCore {
                        side: Side::Buy,
                        post_only: false,
                        time_in_force: TimeInForce::Gtc,
                        extra: (),
                    },
                    price: 100,
                    quantity_policy: QuantityPolicy::Iceberg {
                        visible_quantity: 0,
                        hidden_quantity: 10,
                        replenish_quantity: 10,
                    },
                },
                expected: Err(CommandError::ZeroQuantity),
            },
            Case {
                name: "iceberg zero hidden quantity",
                order: NewLimitOrder {
                    core: NewOrderCore {
                        side: Side::Buy,
                        post_only: false,
                        time_in_force: TimeInForce::Gtc,
                        extra: (),
                    },
                    price: 100,
                    quantity_policy: QuantityPolicy::Iceberg {
                        visible_quantity: 10,
                        hidden_quantity: 0,
                        replenish_quantity: 10,
                    },
                },
                expected: Err(CommandError::IcebergZeroHiddenQuantity),
            },
            Case {
                name: "iceberg zero replenish quantity",
                order: NewLimitOrder {
                    core: NewOrderCore {
                        side: Side::Buy,
                        post_only: false,
                        time_in_force: TimeInForce::Gtc,
                        extra: (),
                    },
                    price: 100,
                    quantity_policy: QuantityPolicy::Iceberg {
                        visible_quantity: 10,
                        hidden_quantity: 10,
                        replenish_quantity: 0,
                    },
                },
                expected: Err(CommandError::IcebergZeroReplenishQuantity),
            },
            Case {
                name: "post-only standard order",
                order: NewLimitOrder {
                    core: NewOrderCore {
                        side: Side::Buy,
                        post_only: true,
                        time_in_force: TimeInForce::Gtc,
                        extra: (),
                    },
                    price: 100,
                    quantity_policy: QuantityPolicy::Standard { quantity: 10 },
                },
                expected: Ok(()),
            },
            Case {
                name: "immediate time in force standard order",
                order: NewLimitOrder {
                    core: NewOrderCore {
                        side: Side::Buy,
                        post_only: false,
                        time_in_force: TimeInForce::Ioc,
                        extra: (),
                    },
                    price: 100,
                    quantity_policy: QuantityPolicy::Standard { quantity: 10 },
                },
                expected: Ok(()),
            },
            Case {
                name: "post-only immediate time in force",
                order: NewLimitOrder {
                    core: NewOrderCore {
                        side: Side::Buy,
                        post_only: true,
                        time_in_force: TimeInForce::Ioc,
                        extra: (),
                    },
                    price: 100,
                    quantity_policy: QuantityPolicy::Standard { quantity: 10 },
                },
                expected: Err(CommandError::PostOnlyImmediateTif),
            },
        ];

        for case in cases {
            let order = case.order;
            match case.expected {
                Ok(()) => assert!(order.validate().is_ok(), "case: {}", case.name),
                Err(e) => assert_eq!(order.validate().unwrap_err(), e, "case: {}", case.name),
            }
        }
    }

    #[test]
    fn test_validate_new_pegged_order() {
        struct Case {
            name: &'static str,
            order: NewPeggedOrder<()>,
            expected: Result<(), CommandError>,
        }

        let cases = [
            Case {
                name: "valid pegged order",
                order: NewPeggedOrder {
                    core: NewOrderCore {
                        side: Side::Buy,
                        post_only: false,
                        time_in_force: TimeInForce::Gtc,
                        extra: (),
                    },
                    peg_reference: PegReference::Market,
                    quantity: 100,
                },
                expected: Ok(()),
            },
            Case {
                name: "zero quantity",
                order: NewPeggedOrder {
                    core: NewOrderCore {
                        side: Side::Buy,
                        post_only: false,
                        time_in_force: TimeInForce::Gtc,
                        extra: (),
                    },
                    peg_reference: PegReference::Market,
                    quantity: 0,
                },
                expected: Err(CommandError::ZeroQuantity),
            },
            Case {
                name: "post-only pegged order",
                order: NewPeggedOrder {
                    core: NewOrderCore {
                        side: Side::Buy,
                        post_only: true,
                        time_in_force: TimeInForce::Gtc,
                        extra: (),
                    },
                    peg_reference: PegReference::Market,
                    quantity: 100,
                },
                expected: Ok(()),
            },
            Case {
                name: "immediate time in force pegged order",
                order: NewPeggedOrder {
                    core: NewOrderCore {
                        side: Side::Buy,
                        post_only: false,
                        time_in_force: TimeInForce::Ioc,
                        extra: (),
                    },
                    peg_reference: PegReference::Market,
                    quantity: 100,
                },
                expected: Ok(()),
            },
            Case {
                name: "post-only immediate time in force",
                order: NewPeggedOrder {
                    core: NewOrderCore {
                        side: Side::Buy,
                        post_only: true,
                        time_in_force: TimeInForce::Ioc,
                        extra: (),
                    },
                    peg_reference: PegReference::Market,
                    quantity: 100,
                },
                expected: Err(CommandError::PostOnlyImmediateTif),
            },
            Case {
                name: "maker only immediate time in force",
                order: NewPeggedOrder {
                    core: NewOrderCore {
                        side: Side::Buy,
                        post_only: false,
                        time_in_force: TimeInForce::Ioc,
                        extra: (),
                    },
                    peg_reference: PegReference::Primary,
                    quantity: 100,
                },
                expected: Err(CommandError::PeggedNonTakerImmediateTif),
            },
        ];

        for case in cases {
            let order = case.order;
            match case.expected {
                Ok(()) => assert!(order.validate().is_ok(), "case: {}", case.name),
                Err(e) => assert_eq!(order.validate().unwrap_err(), e, "case: {}", case.name),
            }
        }
    }
}

use crate::{
    PegReference, Side, TimeInForce,
    command::{CommandError, validation::validate_pegged_order_invariants},
    orders::*,
};

use serde::{Deserialize, Serialize};

/// Represents a command to submit a new order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmitCmd {
    /// The order to submit
    pub order: NewOrder,
}

/// Represents a new order for all order types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NewOrder {
    /// A new market order
    Market(MarketOrderSpec),
    /// A new limit order
    Limit(LimitOrderSpec),
    /// A new pegged order
    Pegged(NewPeggedOrder),
}

/// Represents a new pegged order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewPeggedOrder {
    /// The core order data
    pub core: NewOrderCore,
    /// The peg reference type
    pub peg_reference: PegReference,
    /// The quantity of the order
    pub quantity: u64,
}

impl NewPeggedOrder {
    /// Validate the order
    pub fn validate(&self) -> Result<(), CommandError> {
        validate_pegged_order_invariants(
            self.peg_reference,
            self.quantity,
            self.core.post_only,
            self.core.time_in_force,
        )
    }
}

/// Represents the shared core data for all order types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewOrderCore {
    /// The side of the order
    pub side: Side,
    /// Whether the order is post-only
    pub post_only: bool,
    /// The time in force of the order
    pub time_in_force: TimeInForce,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_new_pegged_order() {
        struct Case {
            name: &'static str,
            order: NewPeggedOrder,
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

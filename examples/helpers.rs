//! Shared helpers for `examples/*.rs`
//!
//! - [`now`]: wall-clock timestamp (production code should use the event record’s time).
//! - [`sequence_number`]: monotonic sequence (production code should use stream offset / row id).
//! - [`target_order_id`]: extracts the affected order id from a successful submit or amend outcome.
//! - [`populate_book`]: populates a book with standard bids (100 down) and asks (110 up).
//!
//! Each example’s module comment describes its scenario; run with `cargo run --example <name>`.

#![allow(dead_code)]

use std::time::{SystemTime, UNIX_EPOCH};

use matchcore::*;

/// Helper function to get the current timestamp
/// In real-world scenarios, the timestamp should be in the input event record.
pub fn now() -> Timestamp {
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    Timestamp(duration.as_secs())
}

/// Helper function to get the current sequence number
/// In real-world scenarios, the sequence number should be the input event record's offset,
/// e.g., offset of the event stream, or row number of the CSV file.
pub fn sequence_number() -> SequenceNumber {
    static mut NEXT_SEQUENCE_NUMBER: SequenceNumber = SequenceNumber(0);

    unsafe {
        let sequence_number = NEXT_SEQUENCE_NUMBER;
        NEXT_SEQUENCE_NUMBER = sequence_number.next();
        sequence_number
    }
}

/// Helper function to get the target order ID from the command outcome
/// Returns `None` if the command was rejected or the order was cancelled
pub fn target_order_id(outcome: &CommandOutcome) -> Option<OrderId> {
    match outcome {
        CommandOutcome::Applied(CommandReport::Submit(command_effects)) => {
            Some(command_effects.target_order().order_id())
        }
        CommandOutcome::Applied(CommandReport::Amend(command_effects)) => {
            Some(command_effects.target_order().order_id())
        }
        _ => None,
    }
}

/// Helper function to populate a book with standard bids (100 down) and asks (110 up)
pub fn populate_book() -> OrderBook {
    let mut book = OrderBook::new("ETH/USD");

    // Submit standard buy orders from the best price to the worst price
    for i in 0..10 {
        book.execute(&Command {
            meta: CommandMeta {
                sequence_number: sequence_number(),
                timestamp: now(),
            },
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Limit(LimitOrder::new(
                    Price(100 - i),
                    QuantityPolicy::Standard {
                        quantity: Quantity(100),
                    },
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                )),
            }),
        });
    }

    // Submit standard sell orders from the best price to the worst price
    for i in 0..10 {
        book.execute(&Command {
            meta: CommandMeta {
                sequence_number: sequence_number(),
                timestamp: now(),
            },
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Limit(LimitOrder::new(
                    Price(110 + i),
                    QuantityPolicy::Standard {
                        quantity: Quantity(100),
                    },
                    OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
                )),
            }),
        });
    }

    book
}

fn main() {}

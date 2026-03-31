//! Helper functions for the examples

#![allow(dead_code)]

use std::time::{SystemTime, UNIX_EPOCH};

use matchcore::{CommandOutcome, CommandReport, OrderId, SequenceNumber, Timestamp};

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
            Some(command_effects.primary_outcome().order_id())
        }
        CommandOutcome::Applied(CommandReport::Amend(command_effects)) => {
            Some(command_effects.primary_outcome().order_id())
        }
        _ => None,
    }
}

fn main() {}

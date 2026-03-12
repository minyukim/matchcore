use std::time::{SystemTime, UNIX_EPOCH};

use matchcore::{SequenceNumber, Timestamp};

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

#[allow(unused)]
fn main() {}

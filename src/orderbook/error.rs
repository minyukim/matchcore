use crate::{SequenceNumber, Timestamp};

use std::fmt;

use serde::{Deserialize, Serialize};

/// Error that occurs during the execution of a command against the order book
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionError {
    /// The sequence number of the command is invalid
    InvalidSequenceNumber {
        /// The expected sequence number
        expected_sequence_number: SequenceNumber,
        /// The received sequence number
        received_sequence_number: SequenceNumber,
    },
    /// The timestamp of the command is invalid
    InvalidTimestamp {
        /// The last seen timestamp
        last_seen_timestamp: Timestamp,
        /// The received timestamp
        received_timestamp: Timestamp,
    },
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionError::InvalidSequenceNumber {
                expected_sequence_number,
                received_sequence_number,
            } => write!(
                f,
                "invalid sequence number: expected {}, received {}",
                expected_sequence_number, received_sequence_number
            ),
            ExecutionError::InvalidTimestamp {
                last_seen_timestamp,
                received_timestamp,
            } => write!(
                f,
                "invalid timestamp: received timestamp {} is before the last seen timestamp {}",
                received_timestamp, last_seen_timestamp
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(
            ExecutionError::InvalidSequenceNumber {
                expected_sequence_number: SequenceNumber(1),
                received_sequence_number: SequenceNumber(2),
            }
            .to_string(),
            "invalid sequence number: expected 1, received 2"
        );
        assert_eq!(
            ExecutionError::InvalidTimestamp {
                last_seen_timestamp: Timestamp(100),
                received_timestamp: Timestamp(10),
            }
            .to_string(),
            "invalid timestamp: received timestamp 10 is before the last seen timestamp 100"
        );
    }
}

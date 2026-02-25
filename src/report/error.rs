use std::fmt;

use serde::{Deserialize, Serialize};

/// Error that occurs during the execution of a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionError {
    /// The sequence number of the command is invalid
    InvalidSequenceNumber {
        /// The expected sequence number
        expected_sequence_number: u64,
        /// The received sequence number
        received_sequence_number: u64,
    },
    /// The timestamp of the command is invalid
    InvalidTimestamp {
        /// The last seen timestamp
        last_seen_timestamp: u64,
        /// The received timestamp
        received_timestamp: u64,
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
                "Invalid sequence number: expected {}, received {}",
                expected_sequence_number, received_sequence_number
            ),
            ExecutionError::InvalidTimestamp {
                last_seen_timestamp,
                received_timestamp,
            } => write!(
                f,
                "Invalid timestamp: received timestamp {} is before the last seen timestamp {}",
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
        {
            assert_eq!(
                ExecutionError::InvalidSequenceNumber {
                    expected_sequence_number: 1,
                    received_sequence_number: 2,
                }
                .to_string(),
                "Invalid sequence number: expected 1, received 2"
            );
        }
        {
            assert_eq!(
                ExecutionError::InvalidTimestamp {
                    last_seen_timestamp: 100,
                    received_timestamp: 10,
                }
                .to_string(),
                "Invalid timestamp: received timestamp 10 is before the last seen timestamp 100"
            );
        }
    }
}

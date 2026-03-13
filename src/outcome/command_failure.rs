use crate::{CommandError, SequenceNumber, Timestamp};

use std::fmt;

use serde::{Deserialize, Serialize};

/// Reason for failing to execute a command
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandFailure {
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
    /// The command is invalid
    InvalidCommand(CommandError),
    /// The order was not found
    OrderNotFound,
}

impl fmt::Display for CommandFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandFailure::InvalidSequenceNumber {
                expected_sequence_number,
                received_sequence_number,
            } => write!(
                f,
                "invalid sequence number: expected {}, received {}",
                expected_sequence_number, received_sequence_number
            ),
            CommandFailure::InvalidTimestamp {
                last_seen_timestamp,
                received_timestamp,
            } => write!(
                f,
                "invalid timestamp: received timestamp {} is before the last seen timestamp {}",
                received_timestamp, last_seen_timestamp
            ),
            CommandFailure::InvalidCommand(e) => write!(f, "invalid command: {e}"),
            CommandFailure::OrderNotFound => write!(f, "order not found"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(
            CommandFailure::InvalidSequenceNumber {
                expected_sequence_number: SequenceNumber(1),
                received_sequence_number: SequenceNumber(2),
            }
            .to_string(),
            "invalid sequence number: expected 1, received 2"
        );
        assert_eq!(
            CommandFailure::InvalidTimestamp {
                last_seen_timestamp: Timestamp(100),
                received_timestamp: Timestamp(10),
            }
            .to_string(),
            "invalid timestamp: received timestamp 10 is before the last seen timestamp 100"
        );
        assert_eq!(
            CommandFailure::InvalidCommand(CommandError::ZeroPrice).to_string(),
            "invalid command: price is zero"
        );
        assert_eq!(CommandFailure::OrderNotFound.to_string(), "order not found");
    }
}

use crate::{
    command::*,
    orderbook::{ExecutionError, OrderBook},
    report::*,
};

impl OrderBook {
    /// Execute a command against the order book
    /// Returns the execution report for the command
    pub fn execute(&mut self, cmd: &Command) -> Result<CommandExecutionReport, ExecutionError> {
        self.handle_command_meta(cmd.meta)?;

        let outcome = match &cmd.kind {
            CommandKind::Submit(submit_cmd) => self.execute_submit(cmd.meta, submit_cmd),
            CommandKind::Amend(amend_cmd) => self.execute_amend(cmd.meta, amend_cmd),
            CommandKind::Cancel(cancel_cmd) => self.execute_cancel(cancel_cmd),
        };

        Ok(CommandExecutionReport::new(cmd.meta, outcome))
    }

    /// Handle the command metadata
    /// Validates the command metadata, and updates the last sequence number and
    /// the last seen timestamp of the order book if the command is valid.
    fn handle_command_meta(&mut self, meta: CommandMeta) -> Result<(), ExecutionError> {
        self.validate_command_meta(meta)?;

        self.last_sequence_number = Some(meta.sequence_number);
        self.last_seen_timestamp = Some(meta.timestamp);

        Ok(())
    }

    /// Validate the command metadata
    fn validate_command_meta(&self, meta: CommandMeta) -> Result<(), ExecutionError> {
        self.validate_sequence_number(meta.sequence_number)?;
        self.validate_timestamp(meta.timestamp)?;
        Ok(())
    }

    /// Validate the sequence number of the command
    fn validate_sequence_number(&self, sequence_number: u64) -> Result<(), ExecutionError> {
        let expected_sequence_number = match self.last_sequence_number {
            Some(last_sequence_number) => last_sequence_number + 1,
            None => 0,
        };
        if sequence_number != expected_sequence_number {
            return Err(ExecutionError::InvalidSequenceNumber {
                expected_sequence_number,
                received_sequence_number: sequence_number,
            });
        }
        Ok(())
    }

    /// Validate the timestamp of the command
    fn validate_timestamp(&self, timestamp: u64) -> Result<(), ExecutionError> {
        if let Some(last_seen_timestamp) = self.last_seen_timestamp
            && timestamp < last_seen_timestamp
        {
            return Err(ExecutionError::InvalidTimestamp {
                last_seen_timestamp,
                received_timestamp: timestamp,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_command_meta() {
        let mut book: OrderBook = OrderBook::new("TEST".to_string());
        assert!(book.last_sequence_number.is_none());
        assert!(book.last_seen_timestamp.is_none());

        // Expected sequence number is 0
        let result = book.handle_command_meta(CommandMeta {
            sequence_number: 1,
            timestamp: 0,
        });
        assert_eq!(
            result.unwrap_err(),
            ExecutionError::InvalidSequenceNumber {
                expected_sequence_number: 0,
                received_sequence_number: 1,
            }
        );
        assert!(book.last_sequence_number.is_none());
        assert!(book.last_seen_timestamp.is_none());

        let result = book.handle_command_meta(CommandMeta {
            sequence_number: 0,
            timestamp: 0,
        });
        assert!(result.is_ok());
        assert_eq!(book.last_sequence_number, Some(0));
        assert_eq!(book.last_seen_timestamp, Some(0));

        // Expected sequence number is 1
        let result = book.handle_command_meta(CommandMeta {
            sequence_number: 0,
            timestamp: 10,
        });
        assert_eq!(
            result.unwrap_err(),
            ExecutionError::InvalidSequenceNumber {
                expected_sequence_number: 1,
                received_sequence_number: 0,
            }
        );
        assert_eq!(book.last_sequence_number, Some(0));
        assert_eq!(book.last_seen_timestamp, Some(0));

        let result = book.handle_command_meta(CommandMeta {
            sequence_number: 1,
            timestamp: 10,
        });
        assert!(result.is_ok());
        assert_eq!(book.last_sequence_number, Some(1));
        assert_eq!(book.last_seen_timestamp, Some(10));

        // Timestamp is before the last seen timestamp
        let result = book.handle_command_meta(CommandMeta {
            sequence_number: 2,
            timestamp: 9,
        });
        assert_eq!(
            result.unwrap_err(),
            ExecutionError::InvalidTimestamp {
                last_seen_timestamp: 10,
                received_timestamp: 9,
            }
        );
        assert_eq!(book.last_sequence_number, Some(1));
        assert_eq!(book.last_seen_timestamp, Some(10));

        // Expected sequence number is 2
        let result = book.handle_command_meta(CommandMeta {
            sequence_number: 3,
            timestamp: 10,
        });
        assert_eq!(
            result.unwrap_err(),
            ExecutionError::InvalidSequenceNumber {
                expected_sequence_number: 2,
                received_sequence_number: 3,
            }
        );
        assert_eq!(book.last_sequence_number, Some(1));
        assert_eq!(book.last_seen_timestamp, Some(10));

        let result = book.handle_command_meta(CommandMeta {
            sequence_number: 2,
            timestamp: 10,
        });
        assert!(result.is_ok());
        assert_eq!(book.last_sequence_number, Some(2));
        assert_eq!(book.last_seen_timestamp, Some(10));
    }
}

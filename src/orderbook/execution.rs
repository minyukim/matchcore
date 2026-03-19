//! Execution of commands against the order book

mod amend;
mod cancel;
mod submit;

use super::OrderBook;
use crate::{SequenceNumber, Timestamp, command::*, outcome::*};

use std::cmp::Reverse;

impl OrderBook {
    /// Execute a command against the order book
    /// Returns the execution report for the command
    pub fn execute(&mut self, cmd: &Command) -> CommandOutcome {
        if let Err(failure) = self.handle_command_meta(cmd.meta) {
            return CommandOutcome::Rejected(failure);
        }

        self.clean_up_expired_orders(cmd.meta.timestamp);

        match &cmd.kind {
            CommandKind::Submit(submit_cmd) => self.execute_submit(cmd.meta, submit_cmd),
            CommandKind::Amend(amend_cmd) => self.execute_amend(cmd.meta, amend_cmd),
            CommandKind::Cancel(cancel_cmd) => self.execute_cancel(cancel_cmd),
        }
    }

    /// Handle the command metadata
    /// Validates the command metadata, and updates the last sequence number and
    /// the last seen timestamp of the order book if the command is valid.
    fn handle_command_meta(&mut self, meta: CommandMeta) -> Result<(), CommandFailure> {
        self.validate_command_meta(meta)?;

        self.last_sequence_number = Some(meta.sequence_number);
        self.last_seen_timestamp = Some(meta.timestamp);

        Ok(())
    }

    /// Validate the command metadata
    fn validate_command_meta(&self, meta: CommandMeta) -> Result<(), CommandFailure> {
        self.validate_sequence_number(meta.sequence_number)?;
        self.validate_timestamp(meta.timestamp)?;
        Ok(())
    }

    /// Validate the sequence number of the command
    fn validate_sequence_number(
        &self,
        sequence_number: SequenceNumber,
    ) -> Result<(), CommandFailure> {
        let expected_sequence_number = match self.last_sequence_number {
            Some(last_sequence_number) => last_sequence_number.next(),
            None => SequenceNumber(0),
        };
        if sequence_number != expected_sequence_number {
            return Err(CommandFailure::InvalidSequenceNumber {
                expected_sequence_number,
                received_sequence_number: sequence_number,
            });
        }
        Ok(())
    }

    /// Validate the timestamp of the command
    fn validate_timestamp(&self, timestamp: Timestamp) -> Result<(), CommandFailure> {
        if let Some(last_seen_timestamp) = self.last_seen_timestamp
            && timestamp < last_seen_timestamp
        {
            return Err(CommandFailure::InvalidTimestamp {
                last_seen_timestamp,
                received_timestamp: timestamp,
            });
        }
        Ok(())
    }

    /// Clean up expired orders
    fn clean_up_expired_orders(&mut self, timestamp: Timestamp) {
        self.clean_up_expired_limit_orders(timestamp);
        self.clean_up_expired_pegged_orders(timestamp);
    }

    /// Clean up expired limit orders
    fn clean_up_expired_limit_orders(&mut self, timestamp: Timestamp) {
        while let Some(Reverse((expires_at, order_id))) = self.limit.expiration_queue.peek() {
            if *expires_at > timestamp {
                break;
            }
            let expired = match self.limit.orders.get(order_id) {
                Some(order) => order.is_expired(timestamp),
                None => {
                    self.limit.expiration_queue.pop();
                    continue;
                }
            };

            // Check if the order is actually expired, as the TIF of the order may have changed
            // since the order was added to the expiration queue
            if expired {
                self.remove_limit_order(*order_id);
            }

            self.limit.expiration_queue.pop();
        }
    }

    /// Clean up expired pegged orders
    fn clean_up_expired_pegged_orders(&mut self, timestamp: Timestamp) {
        while let Some(Reverse((expires_at, order_id))) = self.pegged.expiration_queue.peek() {
            if *expires_at > timestamp {
                break;
            }
            let expired = match self.pegged.orders.get(order_id) {
                Some(order) => order.is_expired(timestamp),
                None => {
                    self.pegged.expiration_queue.pop();
                    continue;
                }
            };

            // Check if the order is actually expired, as the TIF of the order may have changed
            // since the order was added to the expiration queue
            if expired {
                self.remove_pegged_order(*order_id);
            }

            self.pegged.expiration_queue.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        LimitOrder, OrderFlags, OrderId, PegReference, PeggedOrder, Price, Quantity,
        QuantityPolicy, Side, TimeInForce, Timestamp,
    };

    #[test]
    fn test_handle_command_meta() {
        let mut book: OrderBook = OrderBook::new("TEST");
        assert!(book.last_sequence_number.is_none());
        assert!(book.last_seen_timestamp.is_none());

        // Expected sequence number is 0
        let result = book.handle_command_meta(CommandMeta {
            sequence_number: SequenceNumber(1),
            timestamp: Timestamp(0),
        });
        assert_eq!(
            result.unwrap_err(),
            CommandFailure::InvalidSequenceNumber {
                expected_sequence_number: SequenceNumber(0),
                received_sequence_number: SequenceNumber(1),
            }
        );
        assert!(book.last_sequence_number.is_none());
        assert!(book.last_seen_timestamp.is_none());

        let result = book.handle_command_meta(CommandMeta {
            sequence_number: SequenceNumber(0),
            timestamp: Timestamp(0),
        });
        assert!(result.is_ok());
        assert_eq!(book.last_sequence_number, Some(SequenceNumber(0)));
        assert_eq!(book.last_seen_timestamp, Some(Timestamp(0)));

        // Expected sequence number is 1
        let result = book.handle_command_meta(CommandMeta {
            sequence_number: SequenceNumber(0),
            timestamp: Timestamp(10),
        });
        assert_eq!(
            result.unwrap_err(),
            CommandFailure::InvalidSequenceNumber {
                expected_sequence_number: SequenceNumber(1),
                received_sequence_number: SequenceNumber(0),
            }
        );
        assert_eq!(book.last_sequence_number, Some(SequenceNumber(0)));
        assert_eq!(book.last_seen_timestamp, Some(Timestamp(0)));

        let result = book.handle_command_meta(CommandMeta {
            sequence_number: SequenceNumber(1),
            timestamp: Timestamp(10),
        });
        assert!(result.is_ok());
        assert_eq!(book.last_sequence_number, Some(SequenceNumber(1)));
        assert_eq!(book.last_seen_timestamp, Some(Timestamp(10)));

        // Timestamp is before the last seen timestamp
        let result = book.handle_command_meta(CommandMeta {
            sequence_number: SequenceNumber(2),
            timestamp: Timestamp(9),
        });
        assert_eq!(
            result.unwrap_err(),
            CommandFailure::InvalidTimestamp {
                last_seen_timestamp: Timestamp(10),
                received_timestamp: Timestamp(9),
            }
        );
        assert_eq!(book.last_sequence_number, Some(SequenceNumber(1)));
        assert_eq!(book.last_seen_timestamp, Some(Timestamp(10)));

        // Expected sequence number is 2
        let result = book.handle_command_meta(CommandMeta {
            sequence_number: SequenceNumber(3),
            timestamp: Timestamp(10),
        });
        assert_eq!(
            result.unwrap_err(),
            CommandFailure::InvalidSequenceNumber {
                expected_sequence_number: SequenceNumber(2),
                received_sequence_number: SequenceNumber(3),
            }
        );
        assert_eq!(book.last_sequence_number, Some(SequenceNumber(1)));
        assert_eq!(book.last_seen_timestamp, Some(Timestamp(10)));

        let result = book.handle_command_meta(CommandMeta {
            sequence_number: SequenceNumber(2),
            timestamp: Timestamp(10),
        });
        assert!(result.is_ok());
        assert_eq!(book.last_sequence_number, Some(SequenceNumber(2)));
        assert_eq!(book.last_seen_timestamp, Some(Timestamp(10)));
    }

    #[test]
    fn test_clean_up_expired_limit_orders() {
        let mut book: OrderBook = OrderBook::new("TEST");
        assert_eq!(book.limit.bid_levels.len(), 0);
        assert_eq!(book.limit.orders.len(), 0);
        assert_eq!(book.limit.expiration_queue.len(), 0);

        book.add_limit_order(
            SequenceNumber(0),
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(100),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtd(Timestamp(1000))),
            ),
        );
        assert_eq!(book.limit.bid_levels.len(), 1);
        assert_eq!(book.limit.orders.len(), 1);
        assert_eq!(book.limit.expiration_queue.len(), 1);

        book.add_limit_order(
            SequenceNumber(1),
            OrderId(1),
            LimitOrder::new(
                Price(101),
                QuantityPolicy::Standard {
                    quantity: Quantity(100),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtd(Timestamp(1000))),
            ),
        );
        assert_eq!(book.limit.bid_levels.len(), 2);
        assert_eq!(book.limit.orders.len(), 2);
        assert_eq!(book.limit.expiration_queue.len(), 2);

        book.add_limit_order(
            SequenceNumber(2),
            OrderId(2),
            LimitOrder::new(
                Price(101),
                QuantityPolicy::Standard {
                    quantity: Quantity(100),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtd(Timestamp(1001))),
            ),
        );
        assert_eq!(book.limit.bid_levels.len(), 2);
        assert_eq!(book.limit.orders.len(), 3);
        assert_eq!(book.limit.expiration_queue.len(), 3);

        book.add_limit_order(
            SequenceNumber(3),
            OrderId(3),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(100),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtd(Timestamp(1002))),
            ),
        );
        assert_eq!(book.limit.bid_levels.len(), 2);
        assert_eq!(book.limit.orders.len(), 4);
        assert_eq!(book.limit.expiration_queue.len(), 4);

        // No orders should be expired
        book.clean_up_expired_limit_orders(Timestamp(999));
        assert_eq!(book.limit.bid_levels.len(), 2);
        assert_eq!(book.limit.orders.len(), 4);
        assert_eq!(book.limit.expiration_queue.len(), 4);

        // Two orders at GTD 1000 should be expired
        book.clean_up_expired_limit_orders(Timestamp(1000));
        assert_eq!(book.limit.bid_levels.len(), 2);
        assert_eq!(book.limit.orders.len(), 2);
        assert_eq!(book.limit.expiration_queue.len(), 2);

        // Two remaining orders should be expired
        book.clean_up_expired_limit_orders(Timestamp(1002));
        assert_eq!(book.limit.bid_levels.len(), 0);
        assert_eq!(book.limit.orders.len(), 0);
        assert_eq!(book.limit.expiration_queue.len(), 0);
    }

    #[test]
    fn test_clean_up_non_expiring_limit_orders() {
        let mut book: OrderBook = OrderBook::new("TEST");
        assert_eq!(book.limit.bid_levels.len(), 0);
        assert_eq!(book.limit.orders.len(), 0);
        assert_eq!(book.limit.expiration_queue.len(), 0);

        book.add_limit_order(
            SequenceNumber(0),
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(100),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        assert_eq!(book.limit.bid_levels.len(), 1);
        assert_eq!(book.limit.orders.len(), 1);
        assert_eq!(book.limit.expiration_queue.len(), 0);

        book.clean_up_expired_limit_orders(Timestamp(1000));
        assert_eq!(book.limit.bid_levels.len(), 1);
        assert_eq!(book.limit.orders.len(), 1);
        assert_eq!(book.limit.expiration_queue.len(), 0);
    }

    #[test]
    fn test_clean_up_expired_pegged_orders() {
        let mut book: OrderBook = OrderBook::new("TEST");
        for (peg, count) in [
            (PegReference::Primary, 0),
            (PegReference::Market, 0),
            (PegReference::MidPrice, 0),
        ] {
            assert_eq!(book.pegged.bid_levels[peg.as_index()].order_count(), count);
            assert_eq!(book.pegged.ask_levels[peg.as_index()].order_count(), count);
        }
        assert_eq!(book.pegged.orders.len(), 0);
        assert_eq!(book.pegged.expiration_queue.len(), 0);

        book.add_pegged_order(
            SequenceNumber(0),
            OrderId(0),
            PeggedOrder::new(
                PegReference::Primary,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtd(Timestamp(1000))),
            ),
        );
        for (peg, count) in [
            (PegReference::Primary, 1),
            (PegReference::Market, 0),
            (PegReference::MidPrice, 0),
        ] {
            assert_eq!(book.pegged.bid_levels[peg.as_index()].order_count(), count);
        }
        for (peg, count) in [
            (PegReference::Primary, 0),
            (PegReference::Market, 0),
            (PegReference::MidPrice, 0),
        ] {
            assert_eq!(book.pegged.ask_levels[peg.as_index()].order_count(), count);
        }
        assert_eq!(book.pegged.orders.len(), 1);
        assert_eq!(book.pegged.expiration_queue.len(), 1);

        book.add_pegged_order(
            SequenceNumber(1),
            OrderId(1),
            PeggedOrder::new(
                PegReference::Market,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtd(Timestamp(1000))),
            ),
        );
        for (peg, count) in [
            (PegReference::Primary, 1),
            (PegReference::Market, 1),
            (PegReference::MidPrice, 0),
        ] {
            assert_eq!(book.pegged.bid_levels[peg.as_index()].order_count(), count);
        }
        for (peg, count) in [
            (PegReference::Primary, 0),
            (PegReference::Market, 0),
            (PegReference::MidPrice, 0),
        ] {
            assert_eq!(book.pegged.ask_levels[peg.as_index()].order_count(), count);
        }
        assert_eq!(book.pegged.orders.len(), 2);
        assert_eq!(book.pegged.expiration_queue.len(), 2);

        book.add_pegged_order(
            SequenceNumber(2),
            OrderId(2),
            PeggedOrder::new(
                PegReference::MidPrice,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtd(Timestamp(1001))),
            ),
        );
        for (peg, count) in [
            (PegReference::Primary, 1),
            (PegReference::Market, 1),
            (PegReference::MidPrice, 1),
        ] {
            assert_eq!(book.pegged.bid_levels[peg.as_index()].order_count(), count);
        }
        for (peg, count) in [
            (PegReference::Primary, 0),
            (PegReference::Market, 0),
            (PegReference::MidPrice, 0),
        ] {
            assert_eq!(book.pegged.ask_levels[peg.as_index()].order_count(), count);
        }
        assert_eq!(book.pegged.orders.len(), 3);
        assert_eq!(book.pegged.expiration_queue.len(), 3);

        book.add_pegged_order(
            SequenceNumber(3),
            OrderId(3),
            PeggedOrder::new(
                PegReference::Primary,
                Quantity(10),
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtd(Timestamp(1002))),
            ),
        );
        for (peg, count) in [
            (PegReference::Primary, 1),
            (PegReference::Market, 1),
            (PegReference::MidPrice, 1),
        ] {
            assert_eq!(book.pegged.bid_levels[peg.as_index()].order_count(), count);
        }
        for (peg, count) in [
            (PegReference::Primary, 1),
            (PegReference::Market, 0),
            (PegReference::MidPrice, 0),
        ] {
            assert_eq!(book.pegged.ask_levels[peg.as_index()].order_count(), count);
        }
        assert_eq!(book.pegged.orders.len(), 4);
        assert_eq!(book.pegged.expiration_queue.len(), 4);

        // No orders should be expired
        book.clean_up_expired_pegged_orders(Timestamp(999));
        for (peg, count) in [
            (PegReference::Primary, 1),
            (PegReference::Market, 1),
            (PegReference::MidPrice, 1),
        ] {
            assert_eq!(book.pegged.bid_levels[peg.as_index()].order_count(), count);
        }
        for (peg, count) in [
            (PegReference::Primary, 1),
            (PegReference::Market, 0),
            (PegReference::MidPrice, 0),
        ] {
            assert_eq!(book.pegged.ask_levels[peg.as_index()].order_count(), count);
        }
        assert_eq!(book.pegged.orders.len(), 4);
        assert_eq!(book.pegged.expiration_queue.len(), 4);

        // Two orders at GTD 1000 should be expired
        book.clean_up_expired_pegged_orders(Timestamp(1000));
        for (peg, count) in [
            (PegReference::Primary, 0),
            (PegReference::Market, 0),
            (PegReference::MidPrice, 1),
        ] {
            assert_eq!(book.pegged.bid_levels[peg.as_index()].order_count(), count);
        }
        for (peg, count) in [
            (PegReference::Primary, 1),
            (PegReference::Market, 0),
            (PegReference::MidPrice, 0),
        ] {
            assert_eq!(book.pegged.ask_levels[peg.as_index()].order_count(), count);
        }
        assert_eq!(book.pegged.orders.len(), 2);
        assert_eq!(book.pegged.expiration_queue.len(), 2);

        // Two remaining orders should be expired
        book.clean_up_expired_pegged_orders(Timestamp(1002));
        for (peg, count) in [
            (PegReference::Primary, 0),
            (PegReference::Market, 0),
            (PegReference::MidPrice, 0),
        ] {
            assert_eq!(book.pegged.bid_levels[peg.as_index()].order_count(), count);
            assert_eq!(book.pegged.ask_levels[peg.as_index()].order_count(), count);
        }
        assert_eq!(book.pegged.orders.len(), 0);
        assert_eq!(book.pegged.expiration_queue.len(), 0);
    }
}

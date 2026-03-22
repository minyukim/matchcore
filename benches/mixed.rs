//! Benchmarks for a mixed workload of submit/amend/cancel orders in an order book
//!
//! Run: cargo bench --bench benches -- mixed/

use criterion::Criterion;
use std::hint::black_box;

use matchcore::*;

pub fn benches_mixed(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed");

    let n_submit_standard_bids = 100;
    let submit_standard_bid_commands: Vec<Command> = (0..n_submit_standard_bids)
        .map(|i| Command {
            meta: CommandMeta {
                sequence_number: SequenceNumber(i),
                timestamp: Timestamp(i),
            },
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Limit(LimitOrder::new(
                    Price(995 - i % 10),
                    QuantityPolicy::Standard {
                        quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                )),
            }),
        })
        .collect();
    let mut start_sequence_number = n_submit_standard_bids;

    let n_submit_standard_asks = 100;
    let submit_standard_ask_commands: Vec<Command> = (0..n_submit_standard_asks)
        .map(|i| Command {
            meta: CommandMeta {
                sequence_number: SequenceNumber(start_sequence_number + i),
                timestamp: Timestamp(start_sequence_number + i),
            },
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Limit(LimitOrder::new(
                    Price(1005 + i % 10),
                    QuantityPolicy::Standard {
                        quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
                )),
            }),
        })
        .collect();
    start_sequence_number += n_submit_standard_asks;

    let n_amend_standard_bids = 10;
    let amend_standard_bid_commands: Vec<Command> = (0..n_amend_standard_bids)
        .map(|i| Command {
            meta: CommandMeta {
                sequence_number: SequenceNumber(start_sequence_number + i),
                timestamp: Timestamp(start_sequence_number + i),
            },
            kind: CommandKind::Amend(AmendCmd {
                order_id: OrderId(i),
                patch: AmendPatch::Limit(LimitOrderPatch::new().with_quantity_policy(
                    QuantityPolicy::Standard {
                        quantity: Quantity(5),
                    },
                )),
            }),
        })
        .collect();
    start_sequence_number += n_amend_standard_bids;

    let n_amend_standard_asks = 10;
    let amend_standard_ask_commands: Vec<Command> = (0..n_amend_standard_asks)
        .map(|i| Command {
            meta: CommandMeta {
                sequence_number: SequenceNumber(start_sequence_number + i),
                timestamp: Timestamp(start_sequence_number + i),
            },
            kind: CommandKind::Amend(AmendCmd {
                order_id: OrderId(n_submit_standard_bids + i),
                patch: AmendPatch::Limit(LimitOrderPatch::new().with_quantity_policy(
                    QuantityPolicy::Standard {
                        quantity: Quantity(5),
                    },
                )),
            }),
        })
        .collect();
    start_sequence_number += n_amend_standard_asks;

    let n_submit_market_bids = 10;
    let submit_market_bid_commands: Vec<Command> = (0..n_submit_market_bids)
        .map(|i| Command {
            meta: CommandMeta {
                sequence_number: SequenceNumber(start_sequence_number + i),
                timestamp: Timestamp(start_sequence_number + i),
            },
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Market(MarketOrder::new(Quantity(50), Side::Buy, false)),
            }),
        })
        .collect();
    start_sequence_number += n_submit_market_bids;

    let n_submit_market_asks = 10;
    let submit_market_ask_commands: Vec<Command> = (0..n_submit_market_asks)
        .map(|i| Command {
            meta: CommandMeta {
                sequence_number: SequenceNumber(start_sequence_number + i),
                timestamp: Timestamp(start_sequence_number + i),
            },
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Market(MarketOrder::new(Quantity(50), Side::Sell, false)),
            }),
        })
        .collect();
    start_sequence_number += n_submit_market_asks;

    let n_cancel_standard_bids = 10;
    let cancel_standard_bid_commands: Vec<Command> = (0..n_cancel_standard_bids)
        .map(|i| Command {
            meta: CommandMeta {
                sequence_number: SequenceNumber(start_sequence_number + i),
                timestamp: Timestamp(start_sequence_number + i),
            },
            kind: CommandKind::Cancel(CancelCmd {
                order_id: OrderId(n_submit_standard_bids - i),
                order_kind: OrderKind::Limit,
            }),
        })
        .collect();
    start_sequence_number += n_cancel_standard_bids;

    let n_cancel_standard_asks = 10;
    let cancel_standard_ask_commands: Vec<Command> = (0..n_cancel_standard_asks)
        .map(|i| Command {
            meta: CommandMeta {
                sequence_number: SequenceNumber(start_sequence_number + i),
                timestamp: Timestamp(start_sequence_number + i),
            },
            kind: CommandKind::Cancel(CancelCmd {
                order_id: OrderId(n_submit_standard_bids + n_submit_standard_asks - i),
                order_kind: OrderKind::Limit,
            }),
        })
        .collect();

    group.bench_function("submit_amend_match_cancel", |b| {
        b.iter(|| {
            let mut book = OrderBook::new("TEST");
            for command in &submit_standard_bid_commands {
                let outcome = book.execute(black_box(command));
                black_box(outcome);
            }
            for command in &submit_standard_ask_commands {
                let outcome = book.execute(black_box(command));
                black_box(outcome);
            }
            for command in &amend_standard_bid_commands {
                let outcome = book.execute(black_box(command));
                black_box(outcome);
            }
            for command in &amend_standard_ask_commands {
                let outcome = book.execute(black_box(command));
                black_box(outcome);
            }
            for command in &submit_market_bid_commands {
                let outcome = book.execute(black_box(command));
                black_box(outcome);
            }
            for command in &submit_market_ask_commands {
                let outcome = book.execute(black_box(command));
                black_box(outcome);
            }
            for command in &cancel_standard_bid_commands {
                let outcome = book.execute(black_box(command));
                black_box(outcome);
            }
            for command in &cancel_standard_ask_commands {
                let outcome = book.execute(black_box(command));
                black_box(outcome);
            }
            black_box(book);
        })
    });

    group.finish();
}

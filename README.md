[![Crates.io][crates-badge]][crates-url]
[![Downloads][downloads-badge]][crates-url]
[![Build Status][ci-badge]][ci-url]
[![Docs][docs-badge]][docs-url]

[crates-badge]: https://img.shields.io/crates/v/matchcore
[crates-url]: https://crates.io/crates/matchcore
[downloads-badge]: https://img.shields.io/crates/d/matchcore
[ci-badge]: https://img.shields.io/github/actions/workflow/status/minyukim/matchcore/rust-ci.yml
[ci-url]: https://github.com/minyukim/matchcore/actions/workflows/rust-ci.yml
[docs-badge]: https://docs.rs/matchcore/badge.svg
[docs-url]: https://docs.rs/matchcore

# Matchcore

<!-- cargo-reedme: start -->

<!-- cargo-reedme: info-start

    Do not edit this region by hand
    ===============================

    This region was generated from Rust documentation comments by `cargo-reedme` using this command:

        cargo +nightly reedme

    for more info: https://github.com/nik-rev/cargo-reedme

cargo-reedme: info-end -->

**Matchcore** is a high-performance order book and price-time matching engine implemented as a **single-threaded, deterministic, in-memory state machine**.

It is designed for building **low-latency trading systems, exchange simulators, and market-microstructure research tools**.

The architecture follows principles popularized by the [LMAX Architecture](https://martinfowler.com/articles/lmax.html), prioritizing deterministic execution, minimal synchronization, and predictable performance.

## Features

- Price-time priority matching engine
- Deterministic state machine execution
- Single-threaded design for minimal latency
- Efficient in-memory order book
- Support for advanced order types and flags (e.g., iceberg, pegged, time-in-force)
- Designed for integration with event-driven trading systems
- Clear command → outcome model for reproducible execution

## What’s New in v0.4

This release introduces **price-conditional orders** and a revamped **cascading execution model**, enabling richer order semantics and a scalable foundation for complex order interactions.

### Price-Conditional Orders

Adds a new order type that remains inactive until a price condition is satisfied.

- Activated when the market price reaches a specified threshold:
  - **At or above** a trigger price  
  - **At or below** a trigger price  
- On activation, submits a **new order** (market or limit) to the book  
- The activated order is treated as a **fresh submission with new time priority**

This abstraction supports:

- Stop-loss orders  
- Take-profit orders  

#### Execution Model

- Orders are stored in a **price-conditional sub-book**
- Once triggered, they are moved into a **ready queue**
- Execution is integrated with the matching engine via cascading

#### Activation Strategy

Activation depends on whether a last-trade price exists:

##### With last-trade price

- Conditions are evaluated immediately on **submit/amend**
- Orders that already satisfy the condition:
  - Bypass the book
  - Go directly into the **ready queue**
- Remaining orders are organized by trigger price levels and activated via range scans (`drain_levels`)

##### Without last-trade price

- Conditions cannot be evaluated at submission time  
- All orders are placed into a single **pre-trade level**
- On the first trade:
  - Orders are evaluated **in time priority order**
  - Triggered orders are drained via `drain_pre_trade_level_at_price`

### Cascading Execution Model

Introduces a deterministic mechanism to process dependent order activations.

After a primary order execution:

1. Execute all **ready price-conditional orders** triggered by the last-trade price change  
2. Execute an eligible **pegged taker order**  
3. Repeat until no further orders can be executed  

This ensures:

- Deterministic and reproducible behavior  
- Correct ordering of dependent executions  
- Proper handling of chained activations (conditional → pegged → conditional)

## Architecture

The design is heavily inspired by the **LMAX architecture**, a model widely used in low-latency trading systems.

Core principles include:

- **Single-threaded state machine**
- **Event-driven command processing**
- **Deterministic execution**
- **In-memory data structures**

These design choices eliminate synchronization overhead while guaranteeing reproducible behavior.

### Single-threaded

For an order book of a **single instrument**, events must be processed **strictly sequentially**.

Each event mutates the state of the book and the result of one event directly affects the next. Parallelizing matching for the same instrument therefore provides no performance benefit while introducing locking, contention, and complexity.

Running the matching engine on a **single thread** provides several advantages:

- No locks, contention, or synchronization overhead
- Predictable latency
- Simpler correctness guarantees

This does **not** mean the entire application must be single-threaded.

A typical architecture may look like:

```text
Command Reader/Decoder → Ring Buffer → Matchcore Engine → Ring Buffer → Execution Outcome Encoder/Writer
```

Systems can scale horizontally by **sharding instruments across multiple engine threads**.

For example:

```text
Thread 1 → BTC-USD order book
Thread 2 → ETH-USD order book
Thread 3 → SOL-USD order book
```

### Deterministic

Matchcore operates as a **pure deterministic state machine**.

Given:

- The same initial state
- The same sequence of commands

the engine will always produce **exactly the same results**.

This property enables:

- Deterministic replay
- Offline backtesting
- Simulation environments
- Auditability
- Event-sourced architectures

Deterministic execution is particularly valuable for trading systems where correctness and reproducibility are critical.

### In-memory

All state is maintained **entirely in memory**.

The order book, price levels, and internal queues are optimized for fast access and minimal allocations.

This design provides:

- Extremely low latency
- Predictable performance
- Efficient memory access patterns

Persistence and replication are expected to be handled **outside the engine**, typically through event logs and snapshots.

## Core Concepts

Matchcore processes **commands** and produces **outcomes**.

```text
Command → Matchcore Engine → Outcome
```

Commands represent user intent:

- Submit order
- Amend order
- Cancel order

Outcomes describe the result of execution:

- Applied successfully
- Rejected because the command is invalid or cannot be executed in the current state of the order book

Successfully applied commands may also produce:

- Trades
- Order state changes
- Triggered orders

### Example

```rust
use matchcore::*;

let mut book = OrderBook::new("ETH/USD");

let outcome = book.execute(&Command {
    meta: CommandMeta {
        sequence_number: SequenceNumber(0),
        timestamp: Timestamp(1000),
    },
    kind: CommandKind::Submit(SubmitCmd {
        order: NewOrder::Limit(LimitOrder::new(
            Price(100),
            QuantityPolicy::Standard {
                quantity: Quantity(10),
            },
            OrderFlags::new(Side::Buy, false /* post_only */, TimeInForce::Gtc),
        )),
    }),
});

println!("{outcome}");
```

More examples can be found in the [examples](examples) directory.

## Supported Order Features

Matchcore supports the following order types and execution options.

### Types

- **Market Order**: executes immediately against the best available liquidity; optionally supports market-to-limit behavior if not fully filled
- **Limit Order**: executes at the specified price or better
- **Pegged Order**: dynamically reprices based on a reference price (e.g., best bid/ask)
- **Price-Conditional Order**: becomes active when the market price satisfies a specified condition (e.g., at or above a trigger price); a generic category that includes stop-loss and take-profit orders

### Flags

- **Post-Only**: ensures the order adds liquidity only
- **Time-in-Force**: defines order lifetime (e.g., GTC, IOC, FOK, GTD)

### Quantity Policies

- **Standard**: fully visible quantity
- **Iceberg**: partially visible quantity with hidden reserve that replenishes

### Peg References

- **Primary**: pegs to the same-side best price (e.g., best bid for buy)
- **Market**: pegs to the opposite-side best price (e.g., best ask for buy)
- **Mid-Price**: pegs to the midpoint between best bid and best ask

## Performance

Benchmarks are run with [Criterion](https://bheisler.github.io/criterion.rs/book/).

Matchcore is designed for low-latency, single-threaded, deterministic execution.

Representative benchmark results measured on an Apple M4 using Rust stable are shown below.

To run the benchmarks in your environment, run `make bench`.

### Submit

#### Single-order submit

| Benchmark | Time (median) |
| --- | ---: |
| Single standard order into a fresh book | ~104 ns |
| Single iceberg order into a fresh book | ~104 ns |
| Single post-only order into a fresh book | ~104 ns |
| Single good-till-date order into a fresh book | ~116 ns |
| Single pegged order into a fresh book | ~61 ns |
| Single price-conditional order into a fresh book | ~112 ns |
| Single inactive price-conditional stop-limit order | ~130 ns |
| Single active price-conditional stop-limit order | ~142 ns |

#### 10k orders submit

| Benchmark | Time (median) |
| --- | ---: |
| 10k standard orders into a fresh book | ~272.60 µs |
| 10k iceberg orders into a fresh book | ~274.71 µs |
| 10k post-only orders into a fresh book | ~272.47 µs |
| 10k good-till-date orders into a fresh book | ~284.36 µs |
| 10k pegged orders into a fresh book | ~252.08 µs |
| 10k price-conditional orders into a fresh book | ~280.79 µs |
| 10k inactive price-conditional stop-limit orders | ~264.40 µs |
| 10k active price-conditional stop-limit orders | ~540.86 µs |

### Amend

#### Single-order amend

| Benchmark | Time (median) |
| --- | ---: |
| Single order in single-level book quantity decrease | ~775 ns |
| Single order in multi-level book quantity decrease | ~621 ns |
| Single order in single-level book quantity increase | ~814 ns |
| Single order in multi-level book quantity increase | ~665 ns |
| Single order in single-level book price update | ~809 ns |
| Single order in multi-level book price update | ~684 ns |

#### 10k orders amend

| Benchmark | Time (median) |
| --- | ---: |
| 10k orders in single-level book quantity decrease | ~188.42 µs |
| 10k orders in multi-level book quantity decrease | ~160.43 µs |
| 10k orders in single-level book quantity increase | ~211.50 µs |
| 10k orders in multi-level book quantity increase | ~185.59 µs |
| 10k orders in single-level book price update | ~261.89 µs |
| 10k orders in multi-level book price update | ~251.40 µs |

### Cancel

| Benchmark | Time (median) |
| --- | ---: |
| Single order in single-level book cancel | ~789 ns |
| Single order in multi-level book cancel | ~613 ns |
| 10k orders in single-level book cancel | ~138.39 µs |
| 10k orders in multi-level book cancel | ~121.09 µs |

### Matching

#### Single-level standard book

| Match volume | Time (median) |
| --- | ---: |
| 1 | ~475 ns |
| 10 | ~484 ns |
| 100 | ~669 ns |
| 1000 | ~1.71 µs |
| 10000 | ~10.63 µs |

#### Multi-level standard book

| Match volume | Time (median) |
| --- | ---: |
| 1 | ~586 ns |
| 10 | ~592 ns |
| 100 | ~781 ns |
| 1000 | ~1.90 µs |
| 10000 | ~11.33 µs |

#### Single-level iceberg book

| Match volume | Time (median) |
| --- | ---: |
| 1 | ~472 ns |
| 10 | ~556 ns |
| 100 | ~1.12 µs |
| 1000 | ~5.16 µs |
| 10000 | ~39.26 µs |

#### Multi-level iceberg book

| Match volume | Time (median) |
| --- | ---: |
| 1 | ~580 ns |
| 10 | ~666 ns |
| 100 | ~1.23 µs |
| 1000 | ~4.45 µs |
| 10000 | ~36.30 µs |

### Mixed workload

| Benchmark | Time (median) |
| --- | ---: |
| Submit + amend + match + cancel | ~9.77 µs |

### Notes

- Benchmark results depend on CPU, compiler version, benchmark configuration, and system load.
- These figures illustrate the general performance profile of the engine rather than serve as universal guarantees.
- Full Criterion output includes confidence intervals and regression comparisons.

## Next Steps

### Additional Order Features

- Stop orders
- Last-trade peg reference

### Potential Performance Improvements

Currently, the order book stores price levels using `BTreeMap<Price, LevelId>` and `Slab<PriceLevel>`. This design provides:

- **O(log N)** best-price lookup
- **O(log N)** submit operations to locate the corresponding price level
- **O(1)** amend operations (except when amending the order to a different price level)
- **O(1)** cancel operations (except when cancelling the order removes the price level entirely)

where **N** is the number of price levels.

An alternative design is to store prices in `Vec<(Price, LevelId)>`, sorted by price from **worst → best**, which provides:

- **O(1)** best-price lookup
- **O(N)** insertion / deletion when creating or removing price levels

However, in real-world trading scenarios, most activity occurs **near the best price**, meaning the effective search distance is often small. This can make a linear scan competitive with tree-based structures for typical workloads.

<!-- cargo-reedme: end -->

## Makefile

The project uses a Makefile to simplify the development process.

See the [Makefile](Makefile) for more details, or run `make` to see the available commands.

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

## Contribution

Contributions are welcome! If you would like to contribute, please follow these steps:

1. Fork the repository
2. Create a new branch for your changes
3. Make your changes
4. Run all the checks (`make check`)
5. Submit a pull request

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in matchcore by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

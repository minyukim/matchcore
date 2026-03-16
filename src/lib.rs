//! **Matchcore** is a high-performance order book and price-time matching engine implemented as a **single-threaded, deterministic, in-memory state machine**.
//!
//! It is designed for building **low-latency trading systems, exchange simulators, and market-microstructure research tools**.
//!
//! The architecture follows principles popularized by the [LMAX Architecture](https://martinfowler.com/articles/lmax.html), prioritizing deterministic execution, minimal synchronization, and predictable performance.
//!
//! # Features
//!
//! - Price-time priority matching engine
//! - Deterministic state machine execution
//! - Single-threaded design for minimal latency
//! - Efficient in-memory order book
//! - Support for advanced order types and flags (e.g., iceberg, pegged, time-in-force)
//! - Designed for integration with event-driven trading systems
//! - Clear command → outcome model for reproducible execution
//!
//! # Architecture
//!
//! The design is heavily inspired by the **LMAX architecture**, a model widely used in low-latency trading systems.
//!
//! Core principles include:
//!
//! - **Single-threaded state machine**
//! - **Event-driven command processing**
//! - **Deterministic execution**
//! - **In-memory data structures**
//!
//! These design choices eliminate synchronization overhead while guaranteeing reproducible behavior.
//!
//! ## Single-threaded
//!
//! For an order book of a **single instrument**, events must be processed **strictly sequentially**.
//!
//! Each event mutates the state of the book and the result of one event directly affects the next. Parallelizing matching for the same instrument therefore provides no performance benefit while introducing locking, contention, and complexity.
//!
//! Running the matching engine on a **single thread** provides several advantages:
//!
//! - No locks, contention, or synchronization overhead
//! - Predictable latency
//! - Simpler correctness guarantees
//!
//! This does **not** mean the entire application must be single-threaded.
//!
//! A typical architecture may look like:
//!
//! ```text
//! Command Reader/Decoder → Ring Buffer → Matchcore Engine → Ring Buffer → Execution Outcome Encoder/Writer
//! ```
//!
//! Systems can scale horizontally by **sharding instruments across multiple engine threads**.
//!
//! For example:
//!
//! ```text
//! Thread 1 → BTC-USD order book
//! Thread 2 → ETH-USD order book
//! Thread 3 → SOL-USD order book
//! ```
//!
//! ## Deterministic
//!
//! Matchcore operates as a **pure deterministic state machine**.
//!
//! Given:
//!
//! - The same initial state
//! - The same sequence of commands
//!
//! the engine will always produce **exactly the same results**.
//!
//! This property enables:
//!
//! - Deterministic replay
//! - Offline backtesting
//! - Simulation environments
//! - Auditability
//! - Event-sourced architectures
//!
//! Deterministic execution is particularly valuable for trading systems where correctness and reproducibility are critical.
//!
//! ## In-memory
//!
//! All state is maintained **entirely in memory**.
//!
//! The order book, price levels, and internal queues are optimized for fast access and minimal allocations.
//!
//! This design provides:
//!
//! - Extremely low latency
//! - Predictable performance
//! - Efficient memory access patterns
//!
//! Persistence and replication are expected to be handled **outside the engine**, typically through event logs and snapshots.
//!
//! # Core Concepts
//!
//! Matchcore processes **commands** and produces **outcomes**.
//!
//! ```text
//! Command → Matchcore Engine → Outcome
//! ```
//!
//! Commands represent user intent:
//!
//! - Submit order
//! - Amend order
//! - Cancel order
//!
//! Outcomes describe the result of execution:
//!
//! - Applied successfully
//! - Rejected because the command is invalid or cannot be executed in the current state of the order book
//!
//! Successfully applied commands may also produce:
//!
//! - Trades
//! - Order state changes
//! - Triggered orders
//!
//! ## Example
//!
//! ```rust
//! use matchcore::*;
//!
//! let mut book = OrderBook::new("ETH/USD");
//!
//! let outcome = book.execute(&Command {
//!     meta: CommandMeta {
//!         sequence_number: SequenceNumber(0),
//!         timestamp: Timestamp(1000),
//!     },
//!     kind: CommandKind::Submit(SubmitCmd {
//!         order: NewOrder::Limit(LimitOrder::new(
//!             Price(100),
//!             QuantityPolicy::Standard {
//!                 quantity: Quantity(10),
//!             },
//!             OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
//!         )),
//!     }),
//! });
//!
//! println!("{}", outcome);
//! ```
//!
//! More examples can be found in the [examples](examples) directory.
//!
//! # Order Model
//!
//! ## Types
//!
//! - [Market order](src/orders/market.rs)
//! - [Limit order](src/orders/limit.rs)
//! - [Pegged order](src/orders/pegged.rs)
//!
//! ## Flags
//!
//! - [Post-only](src/orders/flags.rs)
//! - [Time-in-force](src/types/time_in_force.rs)
//!
//! ## Quantity Policies
//!
//! - [Standard](src/types/quantity_policy.rs)
//! - [Iceberg](src/types/quantity_policy.rs)
//!
//! ## Peg References
//!
//! - [Primary](src/types/peg_reference.rs)
//! - [Market](src/types/peg_reference.rs)
//! - [Mid-price](src/types/peg_reference.rs)
//!
//! # Performance
//!
//! Benchmarks are run with [Criterion](https://bheisler.github.io/criterion.rs/book/).
//!
//! Matchcore is designed for low-latency, single-threaded, deterministic execution.
//!
//! Representative benchmark results measured on an Apple M4 using Rust stable are shown below.
//!
//! To run the benchmarks in your environment, run `make bench`.
//!
//! ## Submit
//!
//! | Benchmark | Time |
//! | --- | ---: |
//! | Single standard order into a fresh book | ~123 ns |
//! | Single iceberg order into a fresh book | ~120 ns |
//! | Single post-only order into a fresh book | ~119 ns |
//! | Single good-till-date order into a fresh book | ~143 ns |
//! | Single pegged order into a fresh book | ~90 ns |
//! | 10k standard orders into a fresh book | ~352 µs |
//! | 10k iceberg orders into a fresh book | ~355 µs |
//! | 10k post-only orders into a fresh book | ~354 µs |
//! | 10k good-till-date orders into a fresh book | ~371 µs |
//! | 10k pegged orders into a fresh book | ~284 µs |
//!
//! ## Amend
//!
//! | Benchmark | Time |
//! | --- | ---: |
//! | Single order quantity decrease | ~811 ns |
//! | Single order quantity increase | ~886 ns |
//! | Single order price update | ~874 ns |
//! | 10k orders quantity decrease | ~190 µs |
//! | 10k orders quantity increase | ~511 µs |
//! | 10k orders price update | ~559 µs |
//!
//! ## Cancel
//!
//! | Benchmark | Time |
//! | --- | ---: |
//! | Single order cancel | ~905 ns |
//! | 10k orders cancel | ~243 µs |
//!
//! ## Matching
//!
//! ### Single-level standard book
//!
//! | Match volume | Time |
//! | --- | ---: |
//! | 1 | ~488 ns |
//! | 10 | ~497 ns |
//! | 100 | ~1.13 µs |
//! | 1000 | ~5.02 µs |
//! | 10000 | ~26.30 µs |
//!
//! ### Multi-level standard book
//!
//! | Match volume | Time |
//! | --- | ---: |
//! | 1 | ~726 ns |
//! | 10 | ~735 ns |
//! | 100 | ~1.39 µs |
//! | 1000 | ~5.38 µs |
//! | 10000 | ~26.57 µs |
//!
//! ## Mixed workload
//!
//! | Benchmark | Time |
//! | --- | ---: |
//! | Submit + amend + match + cancel | ~14.4 µs |
//!
//! ## Notes
//!
//! - Benchmark results depend on CPU, compiler version, benchmark configuration, and system load.
//! - These figures illustrate the general performance profile of the engine rather than serve as universal guarantees.
//! - Full Criterion output includes confidence intervals and regression comparisons.
//!
//! # Next Steps
//!
//! ## Additional Order Features
//!
//! - Last-trade peg reference
//! - Stop orders
//!
//! ## Potential Performance Improvements
//!
//! Currently, the order book stores price levels using `BTreeMap<Price, PriceLevel>`. This design provides:
//!
//! - **O(log N)** best-price lookup
//! - **O(log N)** submit / amend / cancel operations to locate the corresponding price level
//!
//! where **N** is the number of price levels.
//!
//! Several alternative designs may improve performance.
//!
//! ### 1. Slab-backed price levels
//!
//! Use `Slab<PriceLevel>` and `BTreeMap<Price, LevelIdx>`, and each order holds its `LevelIdx`, allowing direct lookup of its price level. This would reduce the time complexity of the amend/cancel order operations to **O(1)**, except when cancelling the order removes the price level entirely.
//!
//! ### 2. Sorted vector of price levels
//!
//! Store price levels in `Vec<PriceLevel>`, sorted by price from **worst → best**.
//!
//! Trade-offs:
//!
//! - **O(1)** best-price lookup
//! - **O(N)** insertion / deletion when creating or removing price levels
//!
//! However, in real-world trading scenarios, most activity occurs **near the best price**, meaning the effective search distance is often small. This can make a linear scan competitive with tree-based structures for typical workloads.

mod command;
mod orderbook;
mod orders;
mod outcome;
mod types;
mod utils;

pub use command::*;
pub use orderbook::*;
pub use orders::*;
pub use outcome::*;
pub use types::*;

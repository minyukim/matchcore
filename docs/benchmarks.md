## Submit

### Single-order submit

| Benchmark | Time |
| --- | ---: |
| Single standard order into a fresh book | ~110 ns |
| Single iceberg order into a fresh book | ~109 ns |
| Single post-only order into a fresh book | ~108 ns |
| Single good-till-date order into a fresh book | ~124 ns |
| Single pegged order into a fresh book | ~72 ns |

### 10k orders submit

| Benchmark | Time |
| --- | ---: |
| 10k standard orders into a fresh book | ~328.11 µs |
| 10k iceberg orders into a fresh book | ~327.14 µs |
| 10k post-only orders into a fresh book | ~325.93 µs |
| 10k good-till-date orders into a fresh book | ~342.67 µs |
| 10k pegged orders into a fresh book | ~278.09 µs |

## Amend

### Single-order amend

| Benchmark | Time |
| --- | ---: |
| Single order in single-level book quantity decrease | ~886 ns |
| Single order in multi-level book quantity decrease | ~661 ns |
| Single order in single-level book quantity increase | ~879 ns |
| Single order in multi-level book quantity increase | ~692 ns |
| Single order in single-level book price update | ~957 ns |
| Single order in multi-level book price update | ~757 ns |

### 10k orders amend

| Benchmark | Time |
| --- | ---: |
| 10k orders in single-level book quantity decrease | ~193.70 µs |
| 10k orders in multi-level book quantity decrease | ~223.61 µs |
| 10k orders in single-level book quantity increase | ~227.66 µs |
| 10k orders in multi-level book quantity increase | ~240.68 µs |
| 10k orders in single-level book price update | ~627.40 µs |
| 10k orders in multi-level book price update | ~579.55 µs |

## Cancel

| Benchmark | Time |
| --- | ---: |
| Single order in single-level book cancel | ~877 ns |
| Single order in multi-level book cancel | ~685 ns |
| 10k orders in single-level book cancel | ~223.13 µs |
| 10k orders in multi-level book cancel | ~241.98 µs |

## Matching

### Single-level standard book

| Match volume | Time |
| --- | ---: |
| 1 | ~471 ns |
| 10 | ~478 ns |
| 100 | ~1.04 µs |
| 1000 | ~4.46 µs |
| 10000 | ~20.65 µs |

### Multi-level standard book

| Match volume | Time |
| --- | ---: |
| 1 | ~581 ns |
| 10 | ~595 ns |
| 100 | ~1.12 µs |
| 1000 | ~4.04 µs |
| 10000 | ~20.98 µs |

### Single-level iceberg book

| Match volume | Time |
| --- | ---: |
| 1 | ~470 ns |
| 10 | ~668 ns |
| 100 | ~2.15 µs |
| 1000 | ~8.20 µs |
| 10000 | ~65.51 µs |

### Multi-level iceberg book

| Match volume | Time |
| --- | ---: |
| 1 | ~578 ns |
| 10 | ~778 ns |
| 100 | ~2.13 µs |
| 1000 | ~7.85 µs |
| 10000 | ~61.38 µs |

## Mixed workload

| Benchmark | Time |
| --- | ---: |
| Submit + amend + match + cancel | ~12.21 µs |

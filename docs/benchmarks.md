## Submit

### Single-order submit

| Benchmark | Time (median) |
| --- | ---: |
| Single standard order into a fresh book | ~101 ns |
| Single iceberg order into a fresh book | ~103 ns |
| Single post-only order into a fresh book | ~102 ns |
| Single good-till-date order into a fresh book | ~115 ns |
| Single pegged order into a fresh book | ~59 ns |

### 10k orders submit

| Benchmark | Time (median) |
| --- | ---: |
| 10k standard orders into a fresh book | ~362.20 µs |
| 10k iceberg orders into a fresh book | ~361.04 µs |
| 10k post-only orders into a fresh book | ~361.17 µs |
| 10k good-till-date orders into a fresh book | ~378.40 µs |
| 10k pegged orders into a fresh book | ~271.37 µs |

## Amend

### Single-order amend

| Benchmark | Time (median) |
| --- | ---: |
| Single order in single-level book quantity decrease | ~825 ns |
| Single order in multi-level book quantity decrease | ~701 ns |
| Single order in single-level book quantity increase | ~844 ns |
| Single order in multi-level book quantity increase | ~751 ns |
| Single order in single-level book price update | ~879 ns |
| Single order in multi-level book price update | ~769 ns |

### 10k orders amend

| Benchmark | Time (median) |
| --- | ---: |
| 10k orders in single-level book quantity decrease | ~191.24 µs |
| 10k orders in multi-level book quantity decrease | ~180.87 µs |
| 10k orders in single-level book quantity increase | ~205.52 µs |
| 10k orders in multi-level book quantity increase | ~219.19 µs |
| 10k orders in single-level book price update | ~315.16 µs |
| 10k orders in multi-level book price update | ~320.16 µs |

## Cancel

| Benchmark | Time (median) |
| --- | ---: |
| Single order in single-level book cancel | ~819 ns |
| Single order in multi-level book cancel | ~686 ns |
| 10k orders in single-level book cancel | ~211.22 µs |
| 10k orders in multi-level book cancel | ~216.68 µs |

## Matching

### Single-level standard book

| Match volume | Time (median) |
| --- | ---: |
| 1 | ~467 ns |
| 10 | ~476 ns |
| 100 | ~965 ns |
| 1000 | ~3.95 µs |
| 10000 | ~21.13 µs |

### Multi-level standard book

| Match volume | Time (median) |
| --- | ---: |
| 1 | ~580 ns |
| 10 | ~585 ns |
| 100 | ~1.12 µs |
| 1000 | ~4.02 µs |
| 10000 | ~21.72 µs |

### Single-level iceberg book

| Match volume | Time (median) |
| --- | ---: |
| 1 | ~469 ns |
| 10 | ~639 ns |
| 100 | ~1.95 µs |
| 1000 | ~7.98 µs |
| 10000 | ~66.72 µs |

### Multi-level iceberg book

| Match volume | Time (median) |
| --- | ---: |
| 1 | ~575 ns |
| 10 | ~760 ns |
| 100 | ~1.89 µs |
| 1000 | ~7.69 µs |
| 10000 | ~62.70 µs |

## Mixed workload

| Benchmark | Time (median) |
| --- | ---: |
| Submit + amend + match + cancel | ~12.69 µs |

## Submit

### Single-order submit

| Benchmark | Time (median) |
| --- | ---: |
| Single standard order into a fresh book | ~99 ns |
| Single iceberg order into a fresh book | ~100 ns |
| Single post-only order into a fresh book | ~100 ns |
| Single good-till-date order into a fresh book | ~114 ns |
| Single pegged order into a fresh book | ~55 ns |

### 10k orders submit

| Benchmark | Time (median) |
| --- | ---: |
| 10k standard orders into a fresh book | ~306.48 µs |
| 10k iceberg orders into a fresh book | ~307.58 µs |
| 10k post-only orders into a fresh book | ~307.48 µs |
| 10k good-till-date orders into a fresh book | ~322.02 µs |
| 10k pegged orders into a fresh book | ~218.41 µs |

## Amend

### Single-order amend

| Benchmark | Time (median) |
| --- | ---: |
| Single order in single-level book quantity decrease | ~770 ns |
| Single order in multi-level book quantity decrease | ~609 ns |
| Single order in single-level book quantity increase | ~792 ns |
| Single order in multi-level book quantity increase | ~666 ns |
| Single order in single-level book price update | ~816 ns |
| Single order in multi-level book price update | ~671 ns |

### 10k orders amend

| Benchmark | Time (median) |
| --- | ---: |
| 10k orders in single-level book quantity decrease | ~153.35 µs |
| 10k orders in multi-level book quantity decrease | ~130.82 µs |
| 10k orders in single-level book quantity increase | ~165.50 µs |
| 10k orders in multi-level book quantity increase | ~165.83 µs |
| 10k orders in single-level book price update | ~288.02 µs |
| 10k orders in multi-level book price update | ~276.97 µs |

## Cancel

| Benchmark | Time (median) |
| --- | ---: |
| Single order in single-level book cancel | ~792 ns |
| Single order in multi-level book cancel | ~658 ns |
| 10k orders in single-level book cancel | ~127.33 µs |
| 10k orders in multi-level book cancel | ~104.71 µs |

## Matching

### Single-level standard book

| Match volume | Time (median) |
| --- | ---: |
| 1 | ~431 ns |
| 10 | ~441 ns |
| 100 | ~635 ns |
| 1000 | ~1.62 µs |
| 10000 | ~9.98 µs |

### Multi-level standard book

| Match volume | Time (median) |
| --- | ---: |
| 1 | ~545 ns |
| 10 | ~555 ns |
| 100 | ~731 ns |
| 1000 | ~1.77 µs |
| 10000 | ~10.76 µs |

### Single-level iceberg book

| Match volume | Time (median) |
| --- | ---: |
| 1 | ~436 ns |
| 10 | ~524 ns |
| 100 | ~1.10 µs |
| 1000 | ~5.26 µs |
| 10000 | ~38.93 µs |

### Multi-level iceberg book

| Match volume | Time (median) |
| --- | ---: |
| 1 | ~543 ns |
| 10 | ~641 ns |
| 100 | ~1.19 µs |
| 1000 | ~4.32 µs |
| 10000 | ~35.51 µs |

## Mixed workload

| Benchmark | Time (median) |
| --- | ---: |
| Submit + amend + match + cancel | ~9.68 µs |

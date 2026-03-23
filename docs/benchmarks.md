## Submit

### Single-order submit

| Benchmark | Time |
| --- | ---: |
| Single standard order into a fresh book | ~112 ns |
| Single iceberg order into a fresh book | ~110 ns |
| Single post-only order into a fresh book | ~109 ns |
| Single good-till-date order into a fresh book | ~123 ns |
| Single pegged order into a fresh book | ~72 ns |

### 10k orders submit

| Benchmark | Time |
| --- | ---: |
| 10k standard orders into a fresh book | ~349.74 µs |
| 10k iceberg orders into a fresh book | ~350.67 µs |
| 10k post-only orders into a fresh book | ~350.16 µs |
| 10k good-till-date orders into a fresh book | ~367.81 µs |
| 10k pegged orders into a fresh book | ~276.00 µs |

## Amend

### Single-order amend

| Benchmark | Time |
| --- | ---: |
| Single order in single-level book quantity decrease | ~861 ns |
| Single order in multi-level book quantity decrease | ~694 ns |
| Single order in single-level book quantity increase | ~865 ns |
| Single order in multi-level book quantity increase | ~685 ns |
| Single order in single-level book price update | ~959 ns |
| Single order in multi-level book price update | ~745 ns |

### 10k orders amend

| Benchmark | Time |
| --- | ---: |
| 10k orders in single-level book quantity decrease | ~165.54 µs |
| 10k orders in multi-level book quantity decrease | ~179.37 µs |
| 10k orders in single-level book quantity increase | ~183.29 µs |
| 10k orders in multi-level book quantity increase | ~206.40 µs |
| 10k orders in single-level book price update | ~527.47 µs |
| 10k orders in multi-level book price update | ~498.21 µs |

## Cancel

| Benchmark | Time |
| --- | ---: |
| Single order in single-level book cancel | ~880 ns |
| Single order in multi-level book cancel | ~692 ns |
| 10k orders in single-level book cancel | ~224.55 µs |
| 10k orders in multi-level book cancel | ~248.02 µs |

## Matching

### Single-level standard book

| Match volume | Time |
| --- | ---: |
| 1 | ~463 ns |
| 10 | ~470 ns |
| 100 | ~956 ns |
| 1000 | ~3.73 µs |
| 10000 | ~19.96 µs |

### Multi-level standard book

| Match volume | Time |
| --- | ---: |
| 1 | ~578 ns |
| 10 | ~595 ns |
| 100 | ~1.10 µs |
| 1000 | ~3.91 µs |
| 10000 | ~20.62 µs |

### Single-level iceberg book

| Match volume | Time |
| --- | ---: |
| 1 | ~467 ns |
| 10 | ~681 ns |
| 100 | ~2.07 µs |
| 1000 | ~8.56 µs |
| 10000 | ~64.29 µs |

### Multi-level iceberg book

| Match volume | Time |
| --- | ---: |
| 1 | ~582 ns |
| 10 | ~780 ns |
| 100 | ~2.11 µs |
| 1000 | ~7.87 µs |
| 10000 | ~60.73 µs |

## Mixed workload

| Benchmark | Time |
| --- | ---: |
| Submit + amend + match + cancel | ~12.98 µs |

## Submit

### Single-order submit

| Benchmark | Time (median) |
| --- | ---: |
| Single standard order into a fresh book | ~112 ns |
| Single iceberg order into a fresh book | ~111 ns |
| Single post-only order into a fresh book | ~111 ns |
| Single good-till-date order into a fresh book | ~124 ns |
| Single pegged order into a fresh book | ~73 ns |

### 10k orders submit

| Benchmark | Time (median) |
| --- | ---: |
| 10k standard orders into a fresh book | ~336.92 µs |
| 10k iceberg orders into a fresh book | ~333.22 µs |
| 10k post-only orders into a fresh book | ~334.43 µs |
| 10k good-till-date orders into a fresh book | ~348.38 µs |
| 10k pegged orders into a fresh book | ~275.76 µs |

## Amend

### Single-order amend

| Benchmark | Time (median) |
| --- | ---: |
| Single order in single-level book quantity decrease | ~857 ns |
| Single order in multi-level book quantity decrease | ~696 ns |
| Single order in single-level book quantity increase | ~880 ns |
| Single order in multi-level book quantity increase | ~750 ns |
| Single order in single-level book price update | ~949 ns |
| Single order in multi-level book price update | ~777 ns |

### 10k orders amend

| Benchmark | Time (median) |
| --- | ---: |
| 10k orders in single-level book quantity decrease | ~173.83 µs |
| 10k orders in multi-level book quantity decrease | ~165.64 µs |
| 10k orders in single-level book quantity increase | ~189.94 µs |
| 10k orders in multi-level book quantity increase | ~201.68 µs |
| 10k orders in single-level book price update | ~544.52 µs |
| 10k orders in multi-level book price update | ~520.13 µs |

## Cancel

| Benchmark | Time (median) |
| --- | ---: |
| Single order in single-level book cancel | ~867 ns |
| Single order in multi-level book cancel | ~709 ns |
| 10k orders in single-level book cancel | ~229.49 µs |
| 10k orders in multi-level book cancel | ~233.69 µs |

## Matching

### Single-level standard book

| Match volume | Time (median) |
| --- | ---: |
| 1 | ~477 ns |
| 10 | ~484 ns |
| 100 | ~1.04 µs |
| 1000 | ~4.03 µs |
| 10000 | ~21.71 µs |

### Multi-level standard book

| Match volume | Time (median) |
| --- | ---: |
| 1 | ~580 ns |
| 10 | ~596 ns |
| 100 | ~1.12 µs |
| 1000 | ~4.42 µs |
| 10000 | ~22.18 µs |

### Single-level iceberg book

| Match volume | Time (median) |
| --- | ---: |
| 1 | ~476 ns |
| 10 | ~680 ns |
| 100 | ~2.27 µs |
| 1000 | ~8.78 µs |
| 10000 | ~69.19 µs |

### Multi-level iceberg book

| Match volume | Time (median) |
| --- | ---: |
| 1 | ~580 ns |
| 10 | ~771 ns |
| 100 | ~2.29 µs |
| 1000 | ~8.46 µs |
| 10000 | ~68.38 µs |

## Mixed workload

| Benchmark | Time (median) |
| --- | ---: |
| Submit + amend + match + cancel | ~12.29 µs |

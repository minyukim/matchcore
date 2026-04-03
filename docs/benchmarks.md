## Submit

### Single-order submit

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

### 10k orders submit

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

## Amend

### Single-order amend

| Benchmark | Time (median) |
| --- | ---: |
| Single order in single-level book quantity decrease | ~775 ns |
| Single order in multi-level book quantity decrease | ~621 ns |
| Single order in single-level book quantity increase | ~814 ns |
| Single order in multi-level book quantity increase | ~665 ns |
| Single order in single-level book price update | ~809 ns |
| Single order in multi-level book price update | ~684 ns |

### 10k orders amend

| Benchmark | Time (median) |
| --- | ---: |
| 10k orders in single-level book quantity decrease | ~188.42 µs |
| 10k orders in multi-level book quantity decrease | ~160.43 µs |
| 10k orders in single-level book quantity increase | ~211.50 µs |
| 10k orders in multi-level book quantity increase | ~185.59 µs |
| 10k orders in single-level book price update | ~261.89 µs |
| 10k orders in multi-level book price update | ~251.40 µs |

## Cancel

| Benchmark | Time (median) |
| --- | ---: |
| Single order in single-level book cancel | ~789 ns |
| Single order in multi-level book cancel | ~613 ns |
| 10k orders in single-level book cancel | ~138.39 µs |
| 10k orders in multi-level book cancel | ~121.09 µs |

## Matching

### Single-level standard book

| Match volume | Time (median) |
| --- | ---: |
| 1 | ~475 ns |
| 10 | ~484 ns |
| 100 | ~669 ns |
| 1000 | ~1.71 µs |
| 10000 | ~10.63 µs |

### Multi-level standard book

| Match volume | Time (median) |
| --- | ---: |
| 1 | ~586 ns |
| 10 | ~592 ns |
| 100 | ~781 ns |
| 1000 | ~1.90 µs |
| 10000 | ~11.33 µs |

### Single-level iceberg book

| Match volume | Time (median) |
| --- | ---: |
| 1 | ~472 ns |
| 10 | ~556 ns |
| 100 | ~1.12 µs |
| 1000 | ~5.16 µs |
| 10000 | ~39.26 µs |

### Multi-level iceberg book

| Match volume | Time (median) |
| --- | ---: |
| 1 | ~580 ns |
| 10 | ~666 ns |
| 100 | ~1.23 µs |
| 1000 | ~4.45 µs |
| 10000 | ~36.30 µs |

## Mixed workload

| Benchmark | Time (median) |
| --- | ---: |
| Submit + amend + match + cancel | ~9.77 µs |

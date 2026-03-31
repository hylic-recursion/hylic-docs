# Benchmark results

Execution mode comparison across workload profiles. Four modes
form a 2×2 matrix:

|  | Eager fold | Deferred fold (UIO) |
|---|---|---|
| **Sequential traversal** | fused | uio+fused |
| **Parallel traversal** | rayon | uio+rayon |

Nodes are `usize` IDs with an external adjacency list — O(1)
clone, isolating framework overhead from node-type costs.

## Best-of comparison

```
{{#include ../bench-results/bench-table.txt}}
```

## Speedup vs sequential (fused)

```
{{#include ../bench-results/bench-speedup.txt}}
```

## Chart

![Benchmark chart](../bench-results/bench-chart.svg)

## Benchmark source

```rust
{{#include ../../../hylic/benches/par_bench.rs}}
```

## Support code

```rust
{{#include ../../../hylic/benches/bench_support.rs}}
```

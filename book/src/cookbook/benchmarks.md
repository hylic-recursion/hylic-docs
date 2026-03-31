# Benchmark results

Execution mode comparison across workload profiles. Six modes
form a 3×2 matrix — three parallelism approaches, each with
two executor variants:

|  | Direct executor | Lift-based |
|---|---|---|
| **Sequential** | `fused` | — |
| **rayon children** | `rayon` | `parref+rayon`, `eager+rayon` |
| **Lift parallelism (fused traversal)** | — | `parref+fused`, `eager+fused` |

- **fused**: callback-based recursion, zero allocation. The baseline.
- **rayon**: collects children, `par_iter` for sibling parallelism.
- **parref+fused**: `ParLazy` Lift with `Exec::fused` — builds ParRef tree sequentially, eval triggers rayon.
- **parref+rayon**: `ParLazy` Lift with `Exec::rayon` — both traversal and evaluation parallelized.
- **eager+fused**: `ParEager` Lift with `Exec::fused` — builds heap tree sequentially, WorkPool fork-join.
- **eager+rayon**: `ParEager` Lift with `Exec::rayon` — both traversal and fork-join parallelized.

See [Parallel execution](./parallel_execution.md) for how each
approach works, and [Lifts](../design/lifts.md) for the
transformation mechanism behind `parref` and `eager`.

## Heatmap: speedup vs sequential baseline

<iframe src="../bench-results/bench-report.html" width="100%" height="700" style="border:1px solid #444; border-radius:4px;"></iframe>

## Bar chart

![Benchmark chart](../bench-results/bench-chart.svg)

## Speedup table

```
{{#include ../bench-results/bench-speedup.txt}}
```

## Absolute timing table

```
{{#include ../bench-results/bench-table.txt}}
```

## Observations

- **Fused wins for trivial work** (<10µs/node). Zero allocation, no
  thread overhead. Parallelism costs more than it saves.
- **Rayon and eager+rayon dominate for heavy workloads**. The rayon
  executor parallelizes `fold.init` (which runs in Phase 1 of both
  Lifts). When init is the heavy part, the outer executor's
  parallelism matters most.
- **eager+rayon beats rayon on fold-heavy and balanced workloads** —
  the WorkPool's fork-join adds value when Phase 2 accumulate/finalize
  carry significant work.
- **parref+fused and eager+fused show no speedup** — Phase 1 is
  sequential (Exec::fused), and Phase 2 only does accumulate/finalize
  which is trivial in these benchmarks.
- **Deep trees** (branch_factor=2) show less benefit — few siblings
  to parallelize at each level.

## Workload profiles

Nine configurations test different scenarios on a balanced
breadth-first tree (200 nodes, branch factor 8 unless noted):

| Config | Graph work | Fold work | Notes |
|---|---|---|---|
| `0us:overhead` | 0 | 0 | Pure framework overhead |
| `10us:light` | 5k iters | 5k iters | Light computation |
| `100us:graph-heavy` | 100k iters | 5k iters | Graph discovery dominates |
| `100us:fold-heavy` | 5k iters | 100k iters | Fold init dominates |
| `200us:io` | 200µs spin | 5k iters | Simulated I/O latency |
| `200us:balanced` | 100k iters | 100k iters | Equal graph + fold |
| `1ms:heavy` | 500k iters | 500k iters | Heavy computation |
| `200us:deep` | 200µs spin | 5k iters | Deep tree (bf=2) |
| `100us:large500` | 50k iters | 50k iters | 500 nodes, bf=10 |

## Benchmark source

The benchmark harness uses criterion. Each workload config is
prepared once (tree + fold), then all six modes are benchmarked
against it. The WorkPool is created once per config via
`WorkPool::with`, ensuring thread allocation is excluded from timing.

```rust
{{#include ../../../../hylic/benches/par_bench.rs}}
```

## Support code

```rust
{{#include ../../../../hylic/benches/bench_support.rs}}
```

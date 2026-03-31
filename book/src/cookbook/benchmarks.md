# Benchmark results

Three benchmark suites measure hylic's performance from different angles.

## Suite 1: Hylic execution modes

Compares the 6 hylic execution modes against each other across
11 workload scenarios. Each scenario varies the work distribution
across fold phases (init, accumulate, finalize) and tree shape.

| Mode | What it does |
|---|---|
| `hylic-fused` | Callback recursion, zero allocation |
| `hylic-rayon` | Rayon `par_iter` on children |
| `hylic-parref+fused` | ParLazy Lift, fused traversal |
| `hylic-parref+rayon` | ParLazy Lift, rayon traversal |
| `hylic-eager+fused` | ParEager Lift, fused traversal |
| `hylic-eager+rayon` | ParEager Lift, rayon traversal |

See [Parallel execution](./parallel_execution.md) for how each
mode works, and [Lifts](../concepts/transforms.md) for the
transformation mechanism.

## Suite 2: Hylic vs handrolled baselines

Measures hylic's abstraction overhead by comparing against
handrolled implementations that perform the same work:

| Mode | What it does |
|---|---|
| `hand-seq` | Plain recursion on adjacency list |
| `hand-rayon` | Rayon `par_iter` on children |
| `hand-pool` | Manual WorkPool fork-join |

The handrolled baselines call the same work functions (init,
accumulate, finalize) but bypass hylic's Treeish/Exec/Fold
abstractions.

## Suite 3: Module resolution simulation

Simulates real-world dependency graph resolution. Two
implementations of the same problem:

| Mode | What it does |
|---|---|
| `vanilla-seq` | Natural Rust recursion, HashMap registry |
| `vanilla-rayon` | Same, with `par_iter` on deps |
| `hylic-fused` | hylic Fold + Exec::fused |
| `hylic-rayon` | hylic Fold + Exec::rayon |
| `hylic-parref` | ParLazy Lift + Exec::rayon |
| `hylic-eager` | ParEager Lift + WorkPool |

The vanilla versions are written as a day-to-day Rust developer
would — plain recursion with `Vec::collect`, no framework.

## Results

<iframe src="../bench-results/bench-report.html" width="100%" height="900" style="border:1px solid #444; border-radius:4px;"></iframe>

## Workload scenarios

11 scenarios with real-world analogies:

| Moniker | Init | Accum | Finalize | Graph | IO | Tree | Analogy |
|---|---|---|---|---|---|---|---|
| `noop` | 0 | 0 | 0 | 0 | 0 | bf=8 | Framework overhead |
| `hash` | 5k | 1k | 0 | 5k | 0 | bf=8 | Config lookup |
| `parse-lt` | 50k | 5k | 5k | 10k | 0 | bf=8 | Small config parse |
| `parse-hv` | 200k | 10k | 10k | 50k | 0 | bf=8 | Large file parse |
| `aggr` | 5k | 100k | 5k | 5k | 0 | bf=8 | Merging data |
| `xform` | 5k | 5k | 100k | 5k | 0 | bf=8 | Serialization |
| `bal` | 50k | 50k | 50k | 50k | 0 | bf=8 | Equal-cost phases |
| `io` | 5k | 0 | 0 | 0 | 200µs | bf=8 | Network/disk I/O |
| `wide` | 50k | 10k | 10k | 10k | 0 | bf=20 | Wide dependency tree |
| `deep` | 50k | 10k | 10k | 10k | 0 | bf=2 | Deep chain |
| `lg-dense` | 50k | 10k | 10k | 10k | 0 | bf=10, 500n | Large tree |

Each scenario runs at "small" scale (200 nodes) by default.
A "large" variant (2000-5000 nodes) is structurally present
for deeper analysis.

## Text tables

### Hylic modes

```
{{#include ../bench-results/hylic-modes-table.txt}}
```

### Overhead (hylic vs handrolled)

```
{{#include ../bench-results/overhead-table.txt}}
```

### Module simulation

```
{{#include ../bench-results/module-sim-table.txt}}
```

## Observations

- **Fused is fastest for zero work** — zero allocation, no thread overhead.
- **Rayon and parref+rayon dominate for heavy workloads** — rayon's
  work-stealing parallelizes init (the heaviest phase in most scenarios).
- **Handrolled-rayon ≈ hylic-rayon** — hylic's abstraction overhead
  is minimal; the Treeish/Fold/Exec indirection costs ~0-15% on
  realistic workloads.
- **ParEager (eager+rayon) shines on aggregate/transform** — workloads
  where accumulate and finalize carry significant work, because the
  WorkPool's Phase 2 fork-join parallelizes those phases.
- **Vanilla vs hylic in module sim** — vanilla-rayon and hylic-rayon
  are competitive; hylic adds ~10-30% overhead from the abstraction
  layer, offset by the composability gains.

## Benchmark source

### Hylic modes harness

```rust
{{#include ../../../../hylic/benches/bench_hylic_modes.rs}}
```

### Overhead harness

```rust
{{#include ../../../../hylic/benches/bench_vs_handrolled.rs}}
```

### Module simulation harness

```rust
{{#include ../../../../hylic/benches/bench_module_sim.rs}}
```

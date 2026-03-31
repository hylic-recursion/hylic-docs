# Benchmark results

Three benchmark suites measure different aspects of hylic's performance.
All numbers are wall-clock means from criterion (10 samples, 3s measurement).

<link rel="stylesheet" href="../bench-results/bench-style.css">

## Hylic execution modes

Six execution modes compared across 11 workload scenarios — isolating
the effect of executor choice and Lift strategy.

<div id="bench-hylic-modes">Loading...</div>

## Hylic vs handrolled baselines

Measures hylic's abstraction overhead. The handrolled baselines perform
identical work (same init/accumulate/finalize functions) but bypass
Treeish/Exec/Fold — plain recursion on an adjacency list.

<div id="bench-overhead">Loading...</div>

## Module resolution simulation

A realistic scenario: dependency graph resolution with simulated file
parsing and I/O. "Vanilla" versions are natural Rust — recursive
functions with `HashMap` lookup and `Vec::collect`, as you'd write
without any framework.

<div id="bench-module-sim">Loading...</div>

<script>
async function loadBenchFragment(id, file) {
    try {
        const resp = await fetch('../bench-results/' + file);
        if (resp.ok) {
            document.getElementById(id).innerHTML = await resp.text();
        } else {
            document.getElementById(id).textContent = '(benchmark data not available)';
        }
    } catch (e) {
        document.getElementById(id).textContent = '(failed to load: ' + e.message + ')';
    }
}
loadBenchFragment('bench-hylic-modes', 'hylic-modes.html');
loadBenchFragment('bench-overhead', 'overhead.html');
loadBenchFragment('bench-module-sim', 'module-sim.html');
</script>

## Workload scenarios

11 scenarios vary work distribution and tree shape:

| Moniker | Init | Accum | Finalize | Graph | IO | Tree | What it models |
|---|---|---|---|---|---|---|---|
| `noop` | 0 | 0 | 0 | 0 | 0 | bf=8 | Framework overhead |
| `hash` | 5k | 1k | 0 | 5k | 0 | bf=8 | Config key lookup |
| `parse-lt` | 50k | 5k | 5k | 10k | 0 | bf=8 | Small config parse |
| `parse-hv` | 200k | 10k | 10k | 50k | 0 | bf=8 | Large file parse |
| `aggr` | 5k | 100k | 5k | 5k | 0 | bf=8 | Merging data structures |
| `xform` | 5k | 5k | 100k | 5k | 0 | bf=8 | Serialization / codegen |
| `bal` | 50k | 50k | 50k | 50k | 0 | bf=8 | Equal-cost phases |
| `io` | 5k | 0 | 0 | 0 | 200µs | bf=8 | Network / disk I/O |
| `wide` | 50k | 10k | 10k | 10k | 0 | bf=20 | Wide dependency tree |
| `deep` | 50k | 10k | 10k | 10k | 0 | bf=2 | Deep chain |
| `lg-dense` | 50k | 10k | 10k | 10k | 0 | bf=10 | Large tree (500 nodes) |

Work units are `busy_work(N)` iterations — deterministic CPU burn.
All scenarios use 200 nodes at "small" scale. A "large" scale
(2000-5000 nodes) is available for deeper analysis.

## Observations

**Framework overhead is real but small.** On the `noop` scenario (zero
work), hylic's Treeish/Exec indirection adds measurable overhead
compared to handrolled recursion. On any scenario with actual work
(≥10µs per node), the overhead drops below 15%.

**Rayon's work-stealing is effective.** Both `hylic-rayon` and
`hand-rayon` achieve 3-7x speedup on moderate-to-heavy workloads.
hylic doesn't add meaningful overhead on top of rayon.

**ParRef and ParEager shine differently.** ParRef (lazy) benefits
when traversal and fold are both substantial — the double parallelism
(rayon in traversal + rayon in eval) pays off. ParEager (fork-join)
benefits on accumulate-heavy scenarios (`aggr`, `xform`) where its
Phase 2 parallelism matters.

**Vanilla vs hylic in module simulation.** For small graphs with fast
parsing, vanilla-rayon and hylic-rayon are within 15% of each other.
For large graphs with slow parsing, hylic-parref slightly edges out
vanilla-rayon due to better work distribution. The takeaway: hylic's
abstraction cost is comparable to what you'd pay for composability
in any well-structured code — and it gives you Lift-based parallelism,
tracing, and fold composition for free.

**Where hylic is not the right tool.** If your computation is a flat
data-parallel operation (no tree structure), rayon's `par_iter` alone
is simpler and faster. hylic's value is in *recursive* tree
computations where you want the fold/graph/exec separation.

## Text tables

<details>
<summary>Hylic modes (text)</summary>

```
{{#include ../bench-results/hylic-modes.txt}}
```

</details>

<details>
<summary>Overhead — hylic vs handrolled (text)</summary>

```
{{#include ../bench-results/overhead.txt}}
```

</details>

<details>
<summary>Module simulation (text)</summary>

```
{{#include ../bench-results/module-sim.txt}}
```

</details>

## Benchmark source

<details>
<summary>Hylic modes harness</summary>

```rust
{{#include ../../../../hylic/benches/bench_hylic_modes.rs}}
```

</details>

<details>
<summary>Overhead harness</summary>

```rust
{{#include ../../../../hylic/benches/bench_vs_handrolled.rs}}
```

</details>

<details>
<summary>Module simulation harness</summary>

```rust
{{#include ../../../../hylic/benches/bench_module_sim.rs}}
```

</details>

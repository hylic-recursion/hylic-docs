# Benchmark results

Four benchmark suites. All numbers are wall-clock means from
criterion.

<link rel="stylesheet" href="../bench-results/bench-style.css">

## Quick — WIP improvement tracker

Fast-returning subset for tracking performance across code changes.

**Runners** (5): `real.rayon` handrolled baseline, plus four funnel
variants covering both queue axes (PerWorker, Shared) and both
accumulation axes (OnArrival, OnFinalize), all with EveryK\<4\> wake.

**Scenarios** (9): noop, hash, parse-lt, parse-hv, aggr, xform, bal,
wide, graph-hv — selected for showing meaningful variation between
funnel and baseline. Near-parity workloads (io, deep, fin, graph-io,
lg-dense) are excluded for speed.

Two intensity levels and a multi-revision comparison mode:

| Command | Samples | Measure | Time |
|---|---|---|---|
| `make bench-quick-light` | 20 | 5s | ~5 min |
| `make bench-quick-heavy` | 80 | 20s | ~18 min |
| `make bench-quick-light-ab` | 20 | 5s | ~15 min (3 revisions) |
| `make bench-quick-heavy-ab` | 80 | 20s | ~54 min (3 revisions) |

The `-ab` variants run the same benchmark across multiple git
revisions of hylic, archiving results with timestamps. Add revisions
by appending `label=gitref` in the Makefile target.

```
make bench-quick-light
```

<div id="bench-quick">Loading...</div>

## Overhead — framework cost

Fused executor vs handrolled recursive baselines. No parallelism.
Measures the cost of hylic's fold/treeish abstraction.

```
make -C hylic-benchmark bench-overhead
```

<div id="bench-overhead">Loading...</div>

## Matrix — full executor comparison

16 funnel policy variants (4 queue×accumulate × 4 wake) across
14 workload scenarios, plus Rayon, Sheque, and handrolled baselines.

```
make -C hylic-benchmark bench-matrix
```

<div id="bench-matrix">Loading...</div>

<div id="bench-matrix-analysis">Loading...</div>

## Module simulation — realistic workload

Dependency graph resolution with simulated file parsing and I/O.

```
make -C hylic-benchmark bench-modsim
```

<div id="bench-modsim">Loading...</div>

<script>
async function loadBenchFragment(id, file) {
    try {
        const resp = await fetch('../bench-results/' + file);
        if (resp.ok) {
            document.getElementById(id).innerHTML = await resp.text();
        } else {
            document.getElementById(id).textContent = '(benchmark data not available — run the command above)';
        }
    } catch (e) {
        document.getElementById(id).textContent = '(failed to load: ' + e.message + ')';
    }
}
loadBenchFragment('bench-quick', 'quick.html');
loadBenchFragment('bench-overhead', 'overhead.html');
loadBenchFragment('bench-matrix', 'matrix.html');
loadBenchFragment('bench-modsim', 'modsim.html');

// Analysis fragment needs script execution after innerHTML injection
(async function() {
    try {
        const resp = await fetch('../bench-results/matrix-analysis.html');
        if (resp.ok) {
            const html = await resp.text();
            const el = document.getElementById('bench-matrix-analysis');
            el.innerHTML = html;
            // Extract and execute scripts (innerHTML doesn't run them)
            el.querySelectorAll('script').forEach(old => {
                const s = document.createElement('script');
                s.textContent = old.textContent;
                old.replaceWith(s);
            });
        } else {
            document.getElementById('bench-matrix-analysis').textContent = '';
        }
    } catch (e) {
        document.getElementById('bench-matrix-analysis').textContent = '';
    }
})();
</script>

## Workload scenarios

```rust
{{#include ../../../../hylic-benchmark/benches/support/scenario.rs:scenario_catalog}}
```

Work phases are driven by `WorkSpec`:

```rust
{{#include ../../../../hylic-benchmark/benches/support/work.rs:work_spec}}
```

## Funnel policy variants

The benchmark wires all 16 policy variants (4 queue×accumulate × 4 wake):

```rust
{{#include ../../../../hylic-benchmark/benches/support/executor_set.rs:funnel_specs}}
```

See [Funnel policies](../funnel/policies.md) for what each axis
means and when to use which preset.

## Text tables

<details>
<summary>Overhead (text)</summary>

```
{{#include ../bench-results/overhead.txt}}
```

</details>

<details>
<summary>Matrix (text)</summary>

```
{{#include ../bench-results/matrix.txt}}
```

</details>

<details>
<summary>Module simulation (text)</summary>

```
{{#include ../bench-results/modsim.txt}}
```

</details>

## Benchmark source

<details>
<summary>Overhead harness</summary>

```rust
{{#include ../../../../hylic-benchmark/benches/bench_overhead.rs}}
```

</details>

<details>
<summary>Matrix harness</summary>

```rust
{{#include ../../../../hylic-benchmark/benches/bench_matrix.rs}}
```

</details>

<details>
<summary>Module simulation harness</summary>

```rust
{{#include ../../../../hylic-benchmark/benches/bench_modsim.rs}}
```

</details>

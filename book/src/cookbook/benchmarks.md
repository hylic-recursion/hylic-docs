# Benchmark results

Four benchmark suites. All numbers are wall-clock means from
criterion.

<link rel="stylesheet" href="../bench-results/bench-style.css">

## Quick — WIP improvement tracker

Fast-returning subset: `real.rayon` baseline vs two funnel variants
(`pw.arrv.k4`, `sh.arrv.k4`) across all workload scenarios.
40 samples, 10s measurement time.

```
make bench-quick
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

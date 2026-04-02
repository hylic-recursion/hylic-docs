# Benchmark results

Three benchmark suites measure different aspects of hylic's performance.

<link rel="stylesheet" href="../bench-results/bench-style.css">

## Sequential modes

Fused × {Shared, Local, Owned} + Sequential + handrolled baselines.
No parallelism — isolates framework overhead and domain overhead.

<div id="bench-sequential">Loading...</div>

## Parallel modes

Rayon, ParLazy, ParEager, WorkPool + handrolled parallel baselines.
All Shared domain. Measures parallelism strategies.

<div id="bench-parallel">Loading...</div>

## Module resolution simulation

A realistic scenario: dependency graph resolution with simulated file
parsing and I/O. "Vanilla" versions are natural Rust — recursive
functions with `HashMap` lookup and `Vec::collect`.

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
loadBenchFragment('bench-sequential', 'sequential.html');
loadBenchFragment('bench-parallel', 'parallel.html');
loadBenchFragment('bench-module-sim', 'module-sim.html');
</script>

## Workload scenarios

12 scenarios vary work distribution and tree shape:

| Moniker | Init | Accum | Finalize | Graph | IO | Tree | What it models |
|---|---|---|---|---|---|---|---|
| `noop` | 0 | 0 | 0 | 0 | 0 | bf=8 | Framework overhead |
| `hash` | 5k | 1k | 0 | 5k | 0 | bf=8 | Config key lookup |
| `parse-lt` | 50k | 5k | 5k | 10k | 0 | bf=8 | Small config parse |
| `parse-hv` | 200k | 10k | 10k | 50k | 0 | bf=8 | Large file parse |
| `aggr` | 5k | 100k | 5k | 5k | 0 | bf=8 | Merging data structures |
| `xform` | 5k | 5k | 100k | 5k | 0 | bf=8 | Serialization / codegen |
| `fin` | 0 | 0 | 100k | 0 | 0 | bf=8 | Pure finalize (isolates ParEager Phase 2) |
| `bal` | 50k | 50k | 50k | 50k | 0 | bf=8 | Equal-cost phases |
| `io` | 5k | 0 | 0 | 0 | 200µs | bf=8 | Network / disk I/O |
| `wide` | 50k | 10k | 10k | 10k | 0 | bf=20 | Wide dependency tree |
| `deep` | 50k | 10k | 10k | 10k | 0 | bf=2 | Deep chain |
| `lg-dense` | 50k | 10k | 10k | 10k | 0 | bf=10 | Large tree (500 nodes) |

Work units are `busy_work(N)` iterations — deterministic CPU burn.
All scenarios use 200 nodes at "small" scale.

## Text tables

<details>
<summary>Sequential (text)</summary>

```
{{#include ../bench-results/sequential.txt}}
```

</details>

<details>
<summary>Parallel (text)</summary>

```
{{#include ../bench-results/parallel.txt}}
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
<summary>Sequential harness</summary>

```rust
{{#include ../../../../hylic/benches/bench_sequential.rs}}
```

</details>

<details>
<summary>Parallel harness</summary>

```rust
{{#include ../../../../hylic/benches/bench_parallel.rs}}
```

</details>

<details>
<summary>Module simulation harness</summary>

```rust
{{#include ../../../../hylic/benches/bench_module_sim.rs}}
```

</details>

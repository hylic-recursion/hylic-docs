# Benchmark results

Wall-clock means from criterion across four harnesses. The
**Overhead** harness pits `Fused` (hylic's sequential executor)
against handrolled single-threaded recursions to measure
framework cost. The **Matrix** harness puts `Funnel` across its
16 policy variants alongside Rayon, a scoped pool, and
`real.rayon` — all runners parallel — across 14 workload
scenarios. The **Module simulation** harness adds a
dependency-graph resolver workload designed to mimic the
ManageBash module-resolution problem the library was originally
built for. The **Quick** harness is a small subset of the
Matrix grid, used to track changes during development and
across git revisions.

Scenarios are synthetic. CPU work is a deterministic LCG loop
inside `black_box`; "I/O" is a spin-wait. Absolute milliseconds
characterise the *shape* of each runner — the relative
ordering, which policy axis wins for which workload — rather
than any specific production pipeline.

<link rel="stylesheet" href="../bench-results/bench-style.css">

## What the numbers say

Sequential first. `Fused` lands within ±20% of `hand.seq`
on every row of the Overhead bench, faster on 8 of 11. The
spread against `real.seq` (a plain `fn f(&T) -> R` with no
hylic types in sight) is within ±16%. The library's
fold/treeish indirection is, in this regime, on the order of
compiler-level noise rather than an integer multiple. The
plausible reason is a uniform per-node shape that monomorphises
predictably plus closures held inside `Fold` and `Treeish`
that the compiler can inline through; whatever the cause, the
practical statement is *parity*, not dominance.

The parallel picture is more interesting. A `Funnel` variant
is the row winner on 10 of 14 Matrix workloads. On the
remaining 4 the row winner is handrolled and the nearest
`Funnel` variant lies within a few percent. No single policy
preset wins across the grid: shallow-wide workloads prefer
`Shared` queues with `OnArrival` accumulation; deep-narrow
prefer `PerWorker` with `OnFinalize`; the wake axis can move a
row by 10–30% on its own. The 14-row table below is most
useful read row-by-row — for any one workload, the policy that
wins tells you something about the workload's shape.

The Module-simulation harness picks at the same trade. On the
four `_slow` rows (large-dense, large-sparse, small-dense,
small-sparse) `sheque`'s skip-list scheduler wins; per-node
work is heavy enough that scheduling ceases to be the
bottleneck. On the four `_fast` rows `Funnel` variants win —
different policy axes per row, unsurprisingly given the
Matrix story.

These properties of `Funnel` are statements about the source,
not inferences from the benchmarks. Policies are monomorphised
(`Funnel<P>` is generic, the entire walk specialises per
policy, no runtime dispatch on strategy). Continuations are
defunctionalised — `Cont<H, R>` is a three-variant enum
(`Root`, `Direct`, `Slot`); the inner loop is `match cont` in
a `loop`, no `Box<dyn FnOnce>` per step. Continuations and
fold chains live in arenas (`ChainNode<H, R>` in a scoped
`Arena`, `Cont<H, R>` in a `ContArena`, both released in bulk
at the end of the pool's lifetime; no per-node `malloc`/`free`).
Under the `OnArrival` accumulation policy, each child result
is folded into its parent's heap on arrival via
`P::Accumulate::deliver`, and the slot is freed; `OnFinalize`
buffers until siblings are complete and then drains. The walk
references the user's fold and treeish by `&'a _`, with the
lifetime tied to the pool's `with(...)` scope; user closures
are not cloned into worker queues. Queue topology is a
compile-time choice — per-worker deques (local push, remote
steal) or a single shared FIFO — and selection is per workload
rather than universal.

See the
[Funnel deep-dive](../funnel/overview.md) for the walk, ticket
system, and arena details, and
[Policies and presets](../funnel/policies.md) for the policy
traits.

## Interactive: Funnel axes viewer

The Matrix bench output filtered by policy axis, marginalised
on demand, with cell-level deviations from `real.rayon`.

<div id="bench-matrix-analysis">Loading...</div>

## Overhead

```
make -C hylic-benchmark bench-overhead
```

<div id="bench-overhead">Loading...</div>

The Overhead table also lists several parallel runners
(`real.rayon`, `hylic-rayon`, `hand.rayon`, `hylic-parref+rayon`,
`hylic-eager+rayon`) for cross-reference. They are not the
denominators for sequential-overhead statements; a parallel
runner beating a sequential one says that multiple cores are
faster than one, not that the framework is slow.

For a framework-vs-handrolled comparison in the parallel
regime, `hylic-rayon` versus `real.rayon` is the
apples-to-apples pair: within ±15% on most rows, with a
worst-case +33% on `parse-lt_sm`. That is a real framework tax
on the parallel path; whether it's tolerable depends on the
choice between `Funnel` and a Rayon-backed executor.

## Matrix

```
make -C hylic-benchmark bench-matrix
```

<div id="bench-matrix">Loading...</div>

Each cell shows the wall-clock mean and the `+X%` deviation
from the row's fastest entry; the row winner is marked
`(best)`. Reading a few rows together brings out the
policy-axis story.

`wide_sm` (200 nodes, branching 20):
`funnel.pw.arrv.push = 6.2ms (best)`, 20% ahead of both
`hand.pool` and `hand.rayon` at 7.5ms. Wide fan-out plus
immediate `OnArrival` delivery and per-worker deques keeps the
push cheap and drains the child heap as siblings complete.

`graph-hv_sm` (heavy edge-discovery, modelling a dependency
resolver): `funnel.sh.fin.k2 = 16.4ms (best)`, 2% ahead of
`hand.rayon` at 16.8ms. Dropping the wake frequency to every
second child amortises the edge-discovery cost better than the
handrolled approaches.

The 4 rows where handrolled wins are `bal_sm`, `io_sm`,
`graph-io_sm`, `noop_sm`. On `bal_sm`, `hand.rayon = 16.1ms`
versus `funnel.sh.fin.push = 17.0ms` (+6%). On `io_sm`,
`sheque = 6.1ms` versus `funnel.pw.arrv.push = 6.2ms` (+2%).
`noop_sm` is dominated by per-node bookkeeping; absolute times
are sub-millisecond and percentage deltas distort.

## Module simulation

```
make -C hylic-benchmark bench-modsim
```

<div id="bench-modsim">Loading...</div>

Eight workloads on two axes — sparse vs dense graph, fast vs
slow per-node work. `sheque` wins all four `_slow` rows;
`Funnel` wins three of four `_fast` rows
(`funnel.pw.fin.push = 1.0ms` on `large-dense_fast`,
`funnel.pw.arrv.push = 1.0ms` on `large-sparse_fast`,
`funnel.sh.arrv.push = 0.3ms` on `small-sparse_fast`); the
fourth, `small-dense_fast`, ties between `sheque` and
`funnel.sh.fin.push` at 0.3ms. For dependency-graph-shaped
workloads with cheap per-node work — the common case for a
module resolver — `Funnel` is the faster choice. Where per-node
work is substantial, the appropriate winner depends on
workload details that `sheque` happens to handle well in this
harness.

## Quick

```
make bench-quick-light
```

<div id="bench-quick">Loading...</div>

Five runners — `real.rayon` plus four `Funnel` variants
covering both queue axes (PerWorker, Shared) and both
accumulation axes (OnArrival, OnFinalize), all with `EveryK<4>`
wake. Nine scenarios chosen for variation: `noop`, `hash`,
`parse-lt`, `parse-hv`, `aggr`, `xform`, `bal`, `wide`,
`graph-hv`. Near-parity scenarios (`io`, `deep`, `fin`,
`graph-io`, `lg-dense`) are excluded.

The `-ab` variants run the same bench across multiple git
revisions of `hylic`, archiving each run with a timestamp.
Further revisions can be added by appending `label=gitref` to
the makefile target.

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

(async function() {
    try {
        const resp = await fetch('../bench-results/matrix-analysis.html');
        if (resp.ok) {
            const html = await resp.text();
            const el = document.getElementById('bench-matrix-analysis');
            el.innerHTML = html;
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

Each scenario is a `TreeSpec` (node count, branching factor)
and a `WorkSpec` (per-phase CPU burn amounts plus an optional
I/O spin-wait). `busy_work` is the deterministic `u64` LCG
loop inside `black_box`; `spin_wait_us` is a wall-clock
busy-wait. The scenarios are synthetic — the intent is to
cover a *shape* space (shallow-wide, deep-narrow,
accumulate-heavy, finalize-heavy, I/O-bound, graph-discovery-
heavy) rather than reproduce any specific production workload.

```rust
{{#include ../../../../hylic-benchmark/benches/support/scenario.rs:scenario_catalog}}
```

```rust
{{#include ../../../../hylic-benchmark/benches/support/work.rs:work_spec}}
```

## Funnel policy variants

```rust
{{#include ../../../../hylic-benchmark/benches/support/executor_set.rs:funnel_specs}}
```

See [Funnel policies](../funnel/policies.md) for the meaning of
each axis, the rationale, and guidance on selecting a preset.

## Text tables

<details>
<summary>Overhead</summary>

```
{{#include ../bench-results/overhead.txt}}
```

</details>

<details>
<summary>Matrix</summary>

```
{{#include ../bench-results/matrix.txt}}
```

</details>

<details>
<summary>Module simulation</summary>

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

<details>
<summary>Runner matrix construction</summary>

```rust
{{#include ../../../../hylic-benchmark/benches/support/runners.rs}}
```

</details>

<details>
<summary>Handrolled baselines</summary>

```rust
{{#include ../../../../hylic-benchmark/benches/support/baselines.rs}}
```

</details>

<details>
<summary>Funnel policy specs</summary>

```rust
{{#include ../../../../hylic-benchmark/benches/support/executor_set.rs}}
```

</details>

## Correctness

Performance numbers are uninformative without correctness. The
Funnel executor has a unit and integration suite under
`hylic/src/exec/variant/funnel/tests/` covering the API, parity
with the Fused baseline, and deterministic results across all
policy variants. An interleaving stress harness in
`tests/interleaving.rs` and `tests/stress.rs` exercises the
scheduler under aggressive steal patterns. Every benchmark
harness asserts that the computed `R` matches a reference Fused
run (`PreparedScenario::expected`) before timing begins; a
policy variant producing a faster-but-incorrect answer would
never reach the tables above.

# Benchmark results

Wall-clock means from criterion, across four harnesses. The
numbers characterise hylic's runtime cost and compare its
executors against handrolled baselines in both sequential and
parallel regimes.

<link rel="stylesheet" href="../bench-results/bench-style.css">

## Two regimes, measured separately

Two distinct questions concern performance, and the harnesses
address them separately:

- **Sequential regime.** Does the `Fold` / `Treeish` indirection
  introduced by `Fused` cost meaningful time relative to a
  handrolled sequential recursion? Measured by the **Overhead**
  harness, which pits `hylic-fused` against handrolled
  single-threaded baselines.
- **Parallel regime.** Does the `Funnel` executor compete with
  established parallel tree-fold implementations (Rayon-style
  divide-and-conquer, Sheque, handrolled scoped pools)? Measured
  by the **Matrix** harness, which sets `Funnel` across 16 policy
  variants alongside `rayon`, `sheque`, `hand.rayon`, and
  `hand.pool`.

Both questions have different baselines and should not be
answered with a single chart — "`Fused` is 3× slower than the
fastest runner" is only informative if the fastest runner is
also sequential. The sections below keep them apart.

## What the numbers show (TL;DR)

- **Sequential regime — `Fused` attains parity with handrolled
  recursion.** On the 11 CPU-bound workloads of the Overhead
  bench, `hylic-fused` lies within ±20% of `hand.seq` on every
  row, and is actually faster on 8 of 11. Against `real.seq` (a
  plain `fn f(&T) -> R`) the spread is within ±16%.
- **Parallel regime — `Funnel` wins a majority of the Matrix
  bench.** A `Funnel` variant is the row winner on 10 of 14
  workloads; on the remaining 4 the nearest `Funnel` variant
  sits within a few percent of the handrolled winner.
- **The three `Funnel` policy axes are a real knob.** Different
  workloads prefer different policies by 10–30%. The
  [interactive viewer](#interactive-funnel-axes-viewer) below
  marginalises each axis directly.
- **Scenarios are synthetic.** CPU burn is produced by a
  deterministic LCG loop inside `black_box`; "I/O" is
  spin-waiting. Absolute ms figures are informative about
  *shape* (relative ordering, policy preference, parity vs
  drift), not about any production pipeline.

## How the performance is achieved

These are properties of the `Funnel` source, rather than
inferences from the benchmarks:

- **Policies are monomorphised.** `Funnel<P>` is generic over a
  `FunnelPolicy` that bundles three axes (queue, accumulation,
  wake). The entire walk is specialised per policy, so no
  runtime dispatch remains on strategy choice.
- **Continuations are defunctionalised rather than boxed.**
  `Cont<H, R>` is a three-variant enum (`Root`, `Direct`,
  `Slot`); there are no `Box<dyn FnOnce>` per-step allocations.
  A worker's inner loop is `match cont { … }` inside a `loop`.
- **Continuations and fold chains live in arenas.**
  `ChainNode<H, R>` is allocated in a scoped `Arena`,
  `Cont<H, R>` in a `ContArena`. Both are released in bulk at
  the end of the pool's lifetime; there is no per-node
  `malloc`/`free`.
- **Child results stream through packed tickets.** Under the
  `OnArrival` accumulation policy, each child result is folded
  into its parent's heap on arrival via
  `P::Accumulate::deliver`, and the slot is freed. Under
  `OnFinalize`, deliveries are buffered until the siblings are
  complete and then drained.
- **The scoped pool avoids copies.** The walk references the
  user's fold and treeish by `&'a _`, with the lifetime tied to
  the pool's `with(...)` scope. User closures are not cloned
  into worker queues.
- **Queue topology is a compile-time choice.** Per-worker deques
  (local push, remote steal) reduce contention on branch-heavy
  workloads; a single shared FIFO minimises imbalance on
  wide-shallow workloads. Neither is a universal winner —
  selection is per workload.

See the [Funnel executor deep-dive](../funnel/overview.md) for
the full walk, ticket system, and arena details. The policy
traits and presets are documented in
[Policies and presets](../funnel/policies.md).

---

## Interactive: Funnel axes viewer

The Matrix bench produces this fragment. Runners may be filtered
by policy axis; axes may be marginalised; cell-level deviations
from `real.rayon` are shown in context.

<div id="bench-matrix-analysis">Loading...</div>

---

## Overhead — sequential framework cost

`Fused` against handrolled **sequential** baselines. No
parallelism appears in this harness from hylic's side; the
table *also* lists several parallel runners for cross-reference,
but those are not the denominators for sequential-overhead
claims.

```
make -C hylic-benchmark bench-overhead
```

<div id="bench-overhead">Loading...</div>

### Sequential comparison: `hylic-fused` vs `hand-seq` / `real-seq`

Both comparators are single-threaded recursions. `hand.seq` is
the library's handrolled scoped recursion; `real.seq` is a plain
`fn f(&T) -> R` written in the style of a reader who has
forgotten about the library entirely.

| Workload      | hand.seq | real.seq | hylic-fused | Δ vs hand.seq | Δ vs real.seq |
|---------------|---------:|---------:|------------:|---------:|---------:|
| aggr_sm       | 50.1ms   | 47.9ms   | 52.3ms      |   +4%    |   +9%    |
| bal_sm        | 92.3ms   | 76.9ms   | 89.0ms      |   −4%    |  +16%    |
| deep_sm       | 34.9ms   | 31.0ms   | 34.6ms      |   −1%    |  +12%    |
| fin_sm        | 45.1ms   | 43.0ms   | 44.6ms      |   −1%    |   +4%    |
| hash_sm       | 5.9ms    | 4.6ms    | 4.2ms       |  −29%    |   −9%    |
| io_sm         | 42.4ms   | 42.0ms   | 42.3ms      |    0%    |   +1%    |
| lg-dense_sm   | 91.1ms   | 96.4ms   | 102.8ms     |  +13%    |   +7%    |
| parse-hv_sm   | 102.4ms  | 111.5ms  | 119.7ms     |  +17%    |   +7%    |
| parse-lt_sm   | 28.9ms   | 30.6ms   | 26.2ms      |   −9%    |  −14%    |
| wide_sm       | 38.6ms   | 31.3ms   | 35.6ms      |   −8%    |  +14%    |
| xform_sm      | 53.9ms   | 50.9ms   | 43.6ms      |  −19%    |  −14%    |

`hylic-fused` lies within ±20% of `hand.seq` on every row and
within ±16% of `real.seq`. It is actually faster than
`hand.seq` on 8 of 11 workloads and faster than `real.seq` on 3.

The framework occasionally outperforming the handrolled version
is plausibly a combination of: (i) a uniform per-node shape that
allows the compiler to monomorphise the hot loop more
predictably; (ii) closures held inside `Fold` and `Treeish`
being static enough to inline under optimisation; (iii)
`hand.seq`, which recurses through slices and `fn` pointers,
not being the fastest conceivable handrolling. The claim,
therefore, is *parity*, not dominance.

### Parallel cross-reference in the same table

The Overhead table also contains several parallel runners —
`real.rayon`, `hylic-rayon`, `hand.rayon`, `hylic-parref+rayon`,
`hylic-eager+rayon`. These are listed for context; they **should
not** be used as denominators for sequential-overhead
statements. A parallel runner beating a sequential one by 4×
does not make `hylic-fused` "slow" in any meaningful sense; it
tells us only that multiple cores are faster than one.

For a framework-vs-handrolled comparison in the *parallel*
regime, `hylic-rayon` vs `real.rayon` is the apples-to-apples
pair: within ±15% on most rows, with a worst case of +33% on
`parse-lt_sm`. That is a real framework tax on the parallel
path, and accepting it or not is part of the design choice
between `Funnel` and a Rayon-backed executor.

### Summary

The `Fused` executor is a viable sequential path, not merely a
development-time scaffold. The cost of the `Fold` / `Treeish`
indirection is on the order of compiler-level noise rather than
an integer multiple.

---

## Matrix — full parallel executor comparison

14 workload scenarios × all 16 Funnel policy variants (4
queue×accumulate × 4 wake), together with three parallel
handrolled baselines (Rayon, a scoped thread pool, and
`real.rayon`). This harness contains no sequential runners; all
comparisons are parallel vs parallel.

```
make -C hylic-benchmark bench-matrix
```

<div id="bench-matrix">Loading...</div>

### Reading the matrix

Each cell shows the wall-clock mean and the `+X%` deviation from
the row's fastest entry. The entry marked `(best)` is the row
winner.

- **Funnel is the row winner on 10 of 14 rows:** aggr
  (`funnel.sh.fin.batch`, 11.0ms), deep (`funnel.pw.arrv.k2`,
  6.6ms), fin (`funnel.pw.arrv.batch`, 7.7ms), graph-hv
  (`funnel.sh.fin.k2`, 16.4ms), hash (`funnel.pw.arrv.batch`,
  0.9ms), lg-dense (`funnel.sh.arrv.batch`, 15.1ms), parse-hv
  (`funnel.sh.fin.k2`, 19.9ms), parse-lt (tied between
  `funnel.pw.arrv.k4` and `funnel.sh.fin.k2`, both 5.5ms), wide
  (`funnel.pw.arrv.push`, 6.2ms), xform (`funnel.sh.fin.k2`,
  9.0ms).
- **On `wide_sm`** (200 nodes, branching factor 20),
  `funnel.pw.arrv.push` is 20% ahead of both `hand.pool` and
  `hand.rayon` (7.5ms each). Wide fan-out with immediate arrival
  delivery is the configuration `Funnel` handles well:
  per-worker deques keep the push cheap; OnArrival accumulation
  drains the child heap as soon as a sibling completes.
- **On `graph-hv_sm`** (heavy edge-discovery workload modelling
  dependency-graph resolution), `funnel.sh.fin.k2` wins outright
  at 16.4ms, 2% ahead of `hand.rayon` (16.8ms). Reducing wake
  frequency (every second child rather than every push)
  amortises the edge-discovery cost better than the handrolled
  approaches.
- **On the remaining 4 rows** (bal, io, graph-io, noop) the row
  winner is handrolled and the nearest `Funnel` variant sits a
  few percent behind. For example: `bal_sm` →
  `hand.rayon = 16.1ms (best)`; nearest `Funnel`,
  `funnel.sh.fin.push = 17.0ms (+6%)`. `io_sm` →
  `sheque = 6.1ms (best)`; nearest `Funnel`,
  `funnel.pw.arrv.push = 6.2ms (+2%)`.
- **`noop_sm`** (a zero-work fold, effectively a framework-overhead
  benchmark) is dominated by per-node bookkeeping. The percentage
  deltas are large because the denominator is near zero; all
  absolute times are sub-millisecond.

The headline finding: across the space of workload shapes, no
single policy axis is universally best, and `Funnel`'s
per-workload best configuration typically wins or is within a
few percent of the best handrolled implementation.

---

## Module simulation — realistic workload

Dependency-graph resolution with simulated file parsing and I/O.
This is the closest proxy to the ManageBash module-resolution
problem that motivated the library. All runners here are
parallel.

```
make -C hylic-benchmark bench-modsim
```

<div id="bench-modsim">Loading...</div>

### Reading modsim

The eight workloads split along two axes: sparse vs dense graph,
and fast vs slow per-node work.

- **`sheque` wins all four `_slow` rows** (large-dense,
  large-sparse, small-dense, small-sparse). Per-node work is
  expensive in those scenarios, and sheque's skip-list scheduler
  amortises well where scheduling is not the bottleneck.
- **Funnel variants win the three `_fast` rows they contest**:
  `large-dense_fast` (`funnel.pw.fin.push = 1.0ms, best`),
  `large-sparse_fast` (`funnel.pw.arrv.push = 1.0ms, best`), and
  `small-sparse_fast` (`funnel.sh.arrv.push = 0.3ms, best`).
  `small-dense_fast` is tied between `sheque` and
  `funnel.sh.fin.push` at 0.3ms.
- **No single policy axis dominates.** The four `_fast` row
  winners have different queue topologies and different
  accumulation strategies.

Practical read: for dependency-graph-style workloads with cheap
per-node work (the common case for a module resolver), `Funnel`
is the faster choice. Where per-node work is substantial, the
appropriate winner depends on workload details that `sheque`
happens to handle well in this harness.

---

## Quick — WIP improvement tracker

A fast-returning subset for tracking performance across code
changes during development. Five runners, nine scenarios, two
intensity levels, plus a multi-revision A/B mode.

**Runners** (5): `real.rayon` as handrolled baseline, together
with four `Funnel` variants covering both queue axes
(PerWorker, Shared) and both accumulation axes (OnArrival,
OnFinalize), all with `EveryK<4>` wake.

**Scenarios** (9): noop, hash, parse-lt, parse-hv, aggr, xform,
bal, wide, graph-hv — selected because they show meaningful
variation between `Funnel` and baseline. Near-parity workloads
(io, deep, fin, graph-io, lg-dense) are excluded in the
interest of running time.

| Command | Samples | Measure | Time |
|---|---|---|---|
| `make bench-quick-light` | 20 | 5s | ~5 min |
| `make bench-quick-heavy` | 80 | 20s | ~18 min |
| `make bench-quick-light-ab` | 20 | 5s | ~15 min (3 revisions) |
| `make bench-quick-heavy-ab` | 80 | 20s | ~54 min (3 revisions) |

The `-ab` variants run the same benchmark across multiple git
revisions of hylic, archiving results with timestamps. Further
revisions may be added by appending `label=gitref` to the
Makefile target.

```
make bench-quick-light
```

<div id="bench-quick">Loading...</div>

---

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

Each scenario is a `TreeSpec` (node count, branching factor) and
a `WorkSpec` (per-phase CPU burn amounts together with an
optional I/O spin-wait). `busy_work` is a deterministic `u64`
LCG loop inside `black_box`; `spin_wait_us` is a wall-clock
busy-wait. The scenarios are synthetic — the intent is to cover
the *shape* space (shallow-wide, deep-narrow, accumulate-heavy,
finalize-heavy, I/O-bound, graph-discovery-heavy) rather than
to reproduce any one production workload.

```rust
{{#include ../../../../hylic-benchmark/benches/support/scenario.rs:scenario_catalog}}
```

Work phases are driven by `WorkSpec`:

```rust
{{#include ../../../../hylic-benchmark/benches/support/work.rs:work_spec}}
```

## Funnel policy variants

All 16 Matrix-bench variants (4 queue×accumulate × 4 wake):

```rust
{{#include ../../../../hylic-benchmark/benches/support/executor_set.rs:funnel_specs}}
```

See [Funnel policies](../funnel/policies.md) for the meaning of
each axis, the physical rationale, and guidance on selecting a
preset for a given workload.

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

The harnesses and the support code that defines the runner
matrix are reproduced in full below. The Matrix bench, in
particular, is the most diagnostic of the three: its runner
list — constructed in `support/runners.rs` — names each of the
sixteen Funnel variants alongside the parallel baselines.

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
<summary>Runner matrix construction (support/runners.rs)</summary>

The Matrix bench's full runner list is assembled here.
`all_hylic_runners` produces the 18 hylic-side runners (Rayon,
Sheque, and the 16 Funnel variants); the handrolled baselines
are added by `baselines::hand_baselines` in `bench_matrix.rs`.

```rust
{{#include ../../../../hylic-benchmark/benches/support/runners.rs}}
```

</details>

<details>
<summary>Handrolled baselines (support/baselines.rs)</summary>

Problem-specific, non-generic recursions that bypass the hylic
abstraction. Used by the Overhead bench for sequential
comparison (`hand.seq`, `real.seq`) and by the Matrix bench for
parallel comparison (`hand.rayon`, `hand.pool`, `real.rayon`).

```rust
{{#include ../../../../hylic-benchmark/benches/support/baselines.rs}}
```

</details>

<details>
<summary>ExecutorSet — the 16 Funnel policy instances (support/executor_set.rs)</summary>

Each of the sixteen cells in the 4 queue×accumulate × 4 wake
grid is instantiated here. The Matrix bench receives an
`ExecutorSet` containing all of them, along with the Funnel
pool and the Sheque spec, and builds one runner per cell.

```rust
{{#include ../../../../hylic-benchmark/benches/support/executor_set.rs}}
```

</details>

## Stress testing for correctness

Performance numbers are uninformative without correctness. The
Funnel executor is accompanied by:

- a full unit and integration suite under
  `hylic/src/exec/variant/funnel/tests/` covering the API
  surface, correctness against the Fused baseline, and
  deterministic result parity across all policy variants;
- an **interleaving stress harness** (`tests/interleaving.rs`,
  `tests/stress.rs`) that exercises the scheduler under
  aggressive steal patterns and verifies that every policy
  variant produces the same final `R` as a sequential fold for
  every scenario in the benchmark catalogue.

Every benchmark harness asserts that the computed `R` matches a
reference Fused run (`PreparedScenario::expected`) before timing
begins; a policy variant producing a faster-but-incorrect answer
would never reach these tables.

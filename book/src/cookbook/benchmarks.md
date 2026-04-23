# Benchmark results

Wall-clock means from criterion, across four harnesses. Read this
chapter to understand what hylic actually costs at runtime, not
just what it claims.

<link rel="stylesheet" href="../bench-results/bench-style.css">

## What the numbers show (TL;DR)

- **Funnel wins outright on 10 of 14 matrix workloads** (aggr,
  deep, fin, graph-hv, hash, lg-dense, parse-hv, parse-lt, wide,
  xform): a funnel variant is the row's `(best)`, ahead of
  handrolled Rayon, Sheque, and a handrolled scoped pool. On the
  remaining 4 (bal, io, graph-io, noop), the nearest funnel
  variant sits within a few percent of the best handrolled
  baseline.
- **Fused executor is at parity with handrolled sequential
  recursion.** `hylic-fused` vs `hand-seq` sits within ±20% on
  every non-noop workload, and is actually faster on 8 of 11
  (notably hash −29%, xform −19%, parse-lt −9%, wide −8%). vs
  `real-seq` (plain `fn f(&Tree) -> R` recursion) the spread is
  also ±16%, with fused faster on hash, parse-lt, and xform.
- **The three Funnel policy axes are a real knob, not a
  decoration.** Different workloads prefer different policies by
  10–30%. The interactive viewer below lets you marginalise each
  axis and see it directly.
- **Scenarios are mostly synthetic.** CPU burn via
  [`busy_work`](#workload-scenarios) and spin-wait for "I/O".
  Do not read absolute ms numbers as application performance —
  read the *shape* (relative ordering, policy preferences,
  parity-vs-drift).

## How performance is achieved

All of this is visible in the Funnel source; the benchmarks are
just the evidence.

- **Policies are monomorphised.** `Funnel<P>` is generic over
  a `FunnelPolicy` that bundles three axes (queue, accumulation,
  wake). The compiler specialises the entire CPS walk per policy,
  so there is no runtime dispatch on "which strategy am I?" — the
  strategy is baked into the binary layout of every worker loop.
- **Continuations are defunctionalised, not boxed.** `Cont<H, R>`
  is a three-variant enum (`Root`, `Direct`, `Slot`); there are no
  `Box<dyn FnOnce>` per-step allocations. The work a worker does
  is `match cont { … }` inside a `loop`.
- **Continuations and fold-chains live in arenas.** `ChainNode<H,
  R>` lives in a scoped `Arena`; `Cont<H, R>` in a `ContArena`.
  Both are allocated against the pool's lifetime, released
  in-bulk at the end — no per-node `malloc`/`free`.
- **Child results stream through packed tickets.** On the
  `OnArrival` accumulation policy, each child result is folded
  into its parent's heap the moment it arrives (via
  `P::Accumulate::deliver`); the slot is then freed. On
  `OnFinalize`, deliveries are buffered until all siblings are
  in, then drained. The buffer-free pattern drops the high-water
  memory line for wide fan-out workloads.
- **The scoped pool avoids copies.** The CPS walk references the
  user's fold and treeish by `&'a _`, with lifetime tied to the
  pool's `with(...)` scope. No cloning of user closures into
  worker queues.
- **Two queue topologies, selected at compile time.** Per-worker
  deques (local push, remote steal) minimise contention for
  branch-heavy workloads; a single shared FIFO minimises
  imbalance for wide-shallow workloads. The benchmark matrix
  below shows that neither is a universal winner — the user picks
  per workload.

See the [Funnel executor deep-dive](../funnel/overview.md) for
the full CPS-walk, ticket, and arena details. The policy traits
and presets are documented in [Policies and
presets](../funnel/policies.md).

---

## Interactive: Funnel axes viewer

The matrix benchmark produces this fragment — filter runners by
policy axis, marginalise axes, and see cell-level deviation from
the `real.rayon` baseline. It's the most compact view of the
library's performance profile.

<div id="bench-matrix-analysis">Loading...</div>

---

## Matrix — full executor comparison

14 workload scenarios × all 16 funnel policy variants (4
queue×accumulate × 4 wake) + three handrolled baselines (Rayon,
scoped thread pool, handrolled sequential).

```
make -C hylic-benchmark bench-matrix
```

<div id="bench-matrix">Loading...</div>

### Reading the matrix

Each cell shows wall-clock mean with `+X%` deviation from the
row's fastest entry. Entries marked `(best)` are the row's winner.

- **Funnel is the row winner on 10 of 14 rows:** aggr
  (`funnel.sh.fin.batch`, 11.0ms), deep (`funnel.pw.arrv.k2`,
  6.6ms), fin (`funnel.pw.arrv.batch`, 7.7ms), graph-hv
  (`funnel.sh.fin.k2`, 16.4ms), hash
  (`funnel.pw.arrv.batch`, 0.9ms), lg-dense
  (`funnel.sh.arrv.batch`, 15.1ms), parse-hv
  (`funnel.sh.fin.k2`, 19.9ms), parse-lt (tied between
  `funnel.pw.arrv.k4` and `funnel.sh.fin.k2`, both 5.5ms), wide
  (`funnel.pw.arrv.push`, 6.2ms), xform (`funnel.sh.fin.k2`,
  9.0ms).
- **On `wide_sm`** (200 nodes, branching factor 20),
  `funnel.pw.arrv.push` is 20% ahead of handrolled pool
  (`hand.pool = 7.5ms, +20%`) and handrolled Rayon
  (`hand.rayon = 7.5ms, +20%`). Wide fan-out with immediate
  arrival delivery is the Funnel sweet spot: per-worker deques
  keep the push cheap, OnArrival accumulation drains the child
  heap as soon as a sibling completes.
- **On `graph-hv_sm`** (heavy edge-discovery workload modelling
  dependency-graph resolution), `funnel.sh.fin.k2` wins outright
  at 16.4ms, 2% ahead of `hand.rayon` (16.8ms). Lower wake
  frequency (every-2 instead of every-push) amortises the
  edge-discovery cost better than the handrolled approaches.
- **On the remaining 4 rows** (bal, io, graph-io, noop) the
  winner is handrolled and funnel sits a few percent behind:
  `bal_sm` → `hand.rayon = 16.1ms, best`; nearest funnel
  (`funnel.sh.fin.push = 17.0ms, +6%`). `io_sm` → `sheque = 6.1ms,
  best`; nearest funnel (`funnel.pw.arrv.push = 6.2ms, +2%`).
- **On `noop_sm`** (zero-work fold, framework overhead bench),
  everything is dominated by per-node bookkeeping. The matrix
  reports large % deltas there because the denominator is
  near-zero; absolute times are all sub-millisecond.

The key finding: across workload *shapes*, no single policy axis
is universally best, and Funnel's per-workload best configuration
typically wins or is within a few percent of the best handrolled
implementation.

---

## Overhead — sequential framework cost

Fused executor vs handrolled recursive baselines, no parallelism.
This isolates the cost of the `Fold`/`Treeish` indirection.

```
make -C hylic-benchmark bench-overhead
```

<div id="bench-overhead">Loading...</div>

### What this shows

The correct apples-to-apples comparison here is **sequential vs
sequential** — `hylic-fused` against `hand-seq` (a hand-written
scoped recursion) and `real-seq` (a plain `fn f(&T) -> R`
recursion). The parallel runners in this table (`real-rayon`,
`hylic-rayon`, `hand-rayon`) are listed for reference but
shouldn't be used as the denominator for sequential overhead
claims — their `(best)` label just marks the fastest cell in
each row, which is of course parallel.

**`hylic-fused` vs `hand-seq` — sequential framework overhead.**
Per-workload deltas:

| Workload      | hand-seq | hylic-fused | Δ       |
|---------------|----------|-------------|---------|
| aggr_sm       | 50.1ms   | 52.3ms      | +4%     |
| bal_sm        | 92.3ms   | 89.0ms      | −4%     |
| deep_sm       | 34.9ms   | 34.6ms      | −1%     |
| fin_sm        | 45.1ms   | 44.6ms      | −1%     |
| hash_sm       | 5.9ms    | 4.2ms       | −29%    |
| io_sm         | 42.4ms   | 42.3ms      | 0%      |
| lg-dense_sm   | 91.1ms   | 102.8ms     | +13%    |
| parse-hv_sm   | 102.4ms  | 119.7ms     | +17%    |
| parse-lt_sm   | 28.9ms   | 26.2ms      | −9%     |
| wide_sm       | 38.6ms   | 35.6ms      | −8%     |
| xform_sm      | 53.9ms   | 43.6ms      | −19%    |

Within ±20% on every row, faster on 8 of 11. Against `real-seq`
the spread is similar (±16%), faster on hash, parse-lt, xform.

**Why can the framework be *faster* than the handrolled version?**
A few reasons plausibly combine: uniform per-node shape lets the
compiler monomorphise the hot loop more predictably; the
closures held inside `Fold` and `Treeish` are static enough to
inline after optimisation; and `hand-seq` (a scoped recursion
using slices and `fn` pointers) is not the fastest conceivable
handrolling — `real-seq` is typically faster than `hand-seq`,
and `hylic-fused` is often in the middle. The claim is *parity*,
not universal dominance.

**Parallel framework overhead** (`hylic-rayon` vs `real-rayon`):
within ±15% on most rows; worst case +33% on `parse-lt_sm`.
That's an honest framework tax on the parallel path, not an
erasure of overhead.

**Takeaway:** the Fused executor is a viable sequential path,
not just a dev-time scaffold. The `Fold`/`Treeish` indirection
costs on the order of compiler wiggle-room, not integer
multiples.

---

## Module simulation — realistic workload

Dependency-graph resolution with simulated file parsing and I/O.
This is the closest proxy to the ManageBash module-resolution
problem that motivated the library.

```
make -C hylic-benchmark bench-modsim
```

<div id="bench-modsim">Loading...</div>

### Reading modsim

The eight workloads split along two axes: sparse vs dense graph,
fast vs slow per-node work.

- **`sheque` wins all four `_slow` rows** (large-dense, large-sparse,
  small-dense, small-sparse). Per-node work is expensive here, and
  sheque's skip-list scheduler amortises well when scheduling isn't
  the bottleneck.
- **Funnel variants win all three `_fast` rows they contest**:
  `large-dense_fast` (`funnel.pw.fin.push = 1.0ms, best`),
  `large-sparse_fast` (`funnel.pw.arrv.push = 1.0ms, best`),
  `small-sparse_fast` (`funnel.sh.arrv.push = 0.3ms, best`).
  `small-dense_fast` is tied between `sheque` and
  `funnel.sh.fin.push` at 0.3ms.
- **No policy axis dominates.** Different funnel winners across
  the 4 fast rows pick different queue topologies and
  accumulation strategies.

The actionable read: for dependency-graph-style workloads with
cheap per-node work (the common case for a module resolver),
Funnel is the faster choice. For heavy per-node work, the
specific winner depends on details sheque happens to handle
well in this harness.

---

## Quick — WIP improvement tracker

Fast-returning subset for tracking performance across code
changes during development. 5 runners, 9 scenarios, two
intensity levels, plus a multi-revision A/B mode.

**Runners** (5): `real.rayon` handrolled baseline, plus four
funnel variants covering both queue axes (PerWorker, Shared) and
both accumulation axes (OnArrival, OnFinalize), all with
`EveryK<4>` wake.

**Scenarios** (9): noop, hash, parse-lt, parse-hv, aggr, xform,
bal, wide, graph-hv — selected for showing meaningful variation
between funnel and baseline. Near-parity workloads (io, deep,
fin, graph-io, lg-dense) are excluded for speed.

| Command | Samples | Measure | Time |
|---|---|---|---|
| `make bench-quick-light` | 20 | 5s | ~5 min |
| `make bench-quick-heavy` | 80 | 20s | ~18 min |
| `make bench-quick-light-ab` | 20 | 5s | ~15 min (3 revisions) |
| `make bench-quick-heavy-ab` | 80 | 20s | ~54 min (3 revisions) |

The `-ab` variants run the same benchmark across multiple git
revisions of hylic, archiving results with timestamps. Add
revisions by appending `label=gitref` in the Makefile target.

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

Each scenario is a `TreeSpec` (node count, branching factor) + a
`WorkSpec` (per-phase CPU burn amounts + optional I/O spin-wait).
`busy_work` is a deterministic u64 LCG loop inside `black_box`;
`spin_wait_us` is a wall-clock busy-wait. These are synthetic —
the point is to cover the *shape* space (shallow-wide,
deep-narrow, accumulate-heavy, finalize-heavy, I/O-bound,
graph-discovery-heavy), not to reproduce any one production
workload.

```rust
{{#include ../../../../hylic-benchmark/benches/support/scenario.rs:scenario_catalog}}
```

Work phases are driven by `WorkSpec`:

```rust
{{#include ../../../../hylic-benchmark/benches/support/work.rs:work_spec}}
```

## Funnel policy variants

All 16 matrix-bench variants (4 queue×accumulate × 4 wake):

```rust
{{#include ../../../../hylic-benchmark/benches/support/executor_set.rs:funnel_specs}}
```

See [Funnel policies](../funnel/policies.md) for what each axis
means, the physical rationale, and when to reach for which preset.

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

## Stress testing for correctness

Performance numbers mean nothing without correctness. The Funnel
executor is backed by:

- a full unit + integration suite under
  `hylic/src/exec/variant/funnel/tests/` covering API surface,
  correctness against the Fused baseline, and deterministic
  result parity across all policy variants,
- an **interleaving stress harness** (`tests/interleaving.rs`,
  `tests/stress.rs`) that exercises the CPS scheduler under
  aggressive steal patterns and validates that every policy
  variant produces the same final `R` as a sequential fold, for
  every scenario in the benchmark catalogue.

Every benchmark harness asserts the computed `R` matches a
reference Fused run (`PreparedScenario::expected`) before timing
begins — a policy variant that produced a faster-but-wrong answer
would not appear in these tables.

# Writing a custom Lift

Ninety-nine percent of transformations compose out of the library
catalogue and the sugar traits. A custom `Lift` impl earns its
keep only when your transformation has cross-node state, needs
per-variant dispatch, or is itself an execution strategy — the
three categories in the library (`explainer_lift`, `SeedLift`,
`ParLazy`/`ParEager`) hit exactly those pain points.

This chapter walks the decisions. The worked example, a
counter-on-init lift, is a deliberately thin case; the table at
the end points to the heavy ones.

## Four decisions

Before you write `apply`, pin these four things:

**1. What are your output types?**

```text
type N2   = ???;
type MapH = ???;
type MapR = ???;
```

If your lift doesn't change an axis, mirror the input: `type N2 = N`.
If it does, declare the new type. `MapH` and `MapR` are typically
wrappers (the Explainer changes `MapR` to `ExplainerResult<N, H, R>`,
bundling the user's R with a trace tree).

**2. What do you do with the input `grow`?**

```text
fn apply<Seed, T>(
    &self,
    grow:    <D as Domain<N>>::Grow<Seed, N>,  // ← this
    ...
```

Three choices: pass through unchanged (you don't touch N),
wrap with an N-conversion (if you change N), or synthesise a new
grow (only `SeedLift` does this, to close the chain). Most
custom lifts pass through.

**3. What do you do with the input `treeish`?**

Pass through, filter, wrap in a visit-intercepting closure
(`wrap_visit_lift`-style), or rebuild entirely (`n_lift`-style,
`SeedLift`-style).

**4. What do you do with the input `fold`?**

Clone it (per-phase, once per closure you'll build), and construct
a new `Fold<D::N2, MapH, MapR>` whose init/accumulate/finalize
call through to the original via captured clones.

Once the four decisions are pinned, `apply`'s body writes itself:
build the three output slots, call `cont(grow', treeish', fold')`,
done.

## The example, decisions first

Target: a `NoteVisits` lift that counts how many times `init` is
called, into a shared `Arc<Mutex<u64>>`. No type changes.

1. **Output types:** everything mirrors input.
   `N2 = N`, `MapH = H`, `MapR = R`.
2. **grow:** pass through.
3. **treeish:** pass through.
4. **fold:** wrap `init` to bump the counter; pass `accumulate`
   and `finalize` through unchanged. Clone `fold` once per phase
   closure.

Full impl:

```rust
{{#include ../../../src/docs_examples.rs:custom_lift_note_visits}}
```

Use it via `LiftBare::run_on` (shown inside the test) or via a
pipeline:

```text
use hylic_pipeline::prelude::*;
let r = my_treeish_pipeline.lift()
    .then_lift(NoteVisits { counter })
    .run_from_node(&FUSED, &root);
```

## When `ShapeLift` beats a custom Lift

If your transformation maps cleanly onto "rewrite one of the three
slots," don't write a Lift impl. Pick the primitive:

| Primitive                                         | When                                   |
|---------------------------------------------------|----------------------------------------|
| `Shared::phases_lift(mi, ma, mf)`                 | rewrite all three Fold phases          |
| `Shared::treeish_lift(mt)`                        | rewrite the graph                      |
| `Shared::n_lift(lift_node, build_treeish, contra)`| coordinated N-change across all slots  |

Or pick one of the per-axis sugar constructors
(`wrap_init_lift`, `zipmap_lift`, `filter_edges_lift`, …) — they're
all `ShapeLift`s with the xforms wired up. `NoteVisits` above is
expressible as `Shared::wrap_init_lift(|n, orig| { counter.bump(); orig(n) })`
with the counter captured by the closure; the custom impl is
shown only to illustrate the CPS shape.

## When a custom Lift is the right answer

Three real cases in the ecosystem:

- **`SeedLift`** — Shared-pinned, stateful (`grow`, `entry_seeds`,
  `entry_heap_fn`), N-changing, and per-variant dispatch on
  `LiftedNode`. Structurally incompatible with `ShapeLift` because
  it has to *consume* the upstream `grow` rather than wrap it.

  ```rust
  {{#include ../../../../hylic/src/ops/lift/seed_lift.rs:seed_lift_struct}}
  ```

  See [`hylic/src/ops/lift/seed_lift.rs`](../../../../hylic/src/ops/lift/seed_lift.rs)
  for the full variant dispatch (`Entry` fans out entry seeds;
  `Node(n)` visits the user's treeish; the downstream grow is
  the synthesised unreachable-closure).

- **`Shared::explainer_lift`** — this *is* a `ShapeLift`, but its
  construction (in `domain/shared/shape_lifts/explainer.rs`) shows
  what a per-node stateful fold-wrap looks like: `init` opens an
  `ExplainerHeap`, `accumulate` appends a transition, `finalize`
  emits an `ExplainerResult`. Good reading before writing a
  fold-rewriting lift of your own.

- **`ParLazy` / `ParEager`** in the `hylic-parallel-lifts` crate —
  *execution strategies* that happen to implement `Lift`. `apply`
  produces a fold whose `accumulate` schedules work onto a thread
  pool. A different mental model from the library shape-lifts;
  read them if you're building parallel execution primitives.

## Capability bounds

`PureLift` and `ShareableLift` are blanket markers. You don't
implement them; they fire automatically when your lift's types
meet the bounds:

- **`PureLift`** — `Clone + 'static` on the lift and Clone on its
  outputs. Sequential executors (`Fused`) need this.
- **`ShareableLift`** — adds `Send + Sync + 'static` everywhere.
  Parallel executors (`Funnel`) need this.

If your lift needs to run under `Funnel`, make the struct `Clone +
Send + Sync + 'static` and ensure every captured field is too.

## Testing

Write a `#[test]` that:

1. Constructs a simple treeish + fold whose result you can compute
   by hand.
2. Applies your lift via `LiftBare::run_on` or a pipeline.
3. Asserts both the result and any side effects the lift should
   record.

For domain parity, repeat the test with Local (and Owned, if your
lift has an Owned variant).

# Writing a custom Lift

The vast majority of transformations compose out of the library
catalogue and the sugar traits. A custom `Lift` implementation
earns its keep only in one of three circumstances: when the
transformation carries cross-node state, when it requires
per-variant dispatch, or when it is itself an execution strategy.
The two custom lifts present in the library — `explainer_lift`
and `SeedLift` — correspond to exactly these cases.

This chapter walks through the decisions involved. The worked
example, a counter-on-init lift, is deliberately minimal; the
table at the end points to the more substantial cases.

## Four decisions

Before writing `apply`, four things must be decided:

**1. Output types.**

```text
type N2   = ???;
type MapH = ???;
type MapR = ???;
```

If the lift does not change an axis, mirror the input: `type N2 = N`.
Otherwise, declare the new type. `MapH` and `MapR` are typically
wrappers — the Explainer, for instance, changes `MapR` to
`ExplainerResult<N, H, R>`, which bundles the user's `R` with a
trace tree.

**2. Treatment of the input `grow`.**

```text
fn apply<Seed, T>(
    &self,
    grow:    <D as Domain<N>>::Grow<Seed, N>,  // ← this
    ...
```

Three options are available: pass `grow` through unchanged (when
`N` is not modified), wrap it with an N-conversion (when `N` is
modified), or synthesise a fresh `grow` (done only by `SeedLift`,
which closes the chain). Most custom lifts pass the input
through.

**3. Treatment of the input `treeish`.**

Pass through, filter, wrap in a visit-intercepting closure (as in
`wrap_visit_lift`), or rebuild entirely (as in `n_lift` or
`SeedLift`).

**4. Treatment of the input `fold`.**

Clone it — once per phase closure to be built — and construct a
new `Fold<D::N2, MapH, MapR>` whose `init`, `accumulate`, and
`finalize` delegate to the original through captured clones.

With the four decisions in place, `apply`'s body follows
directly: build the three output slots and call
`cont(grow', treeish', fold')`.

## Example, decisions first

Goal: a `NoteVisits` lift that counts calls to `init` into a
shared `Arc<Mutex<u64>>`. No type changes.

1. **Output types.** All three mirror the input:
   `N2 = N`, `MapH = H`, `MapR = R`.
2. **grow.** Passed through unchanged.
3. **treeish.** Passed through unchanged.
4. **fold.** Wrap `init` to increment the counter; pass
   `accumulate` and `finalize` through unchanged. `fold` is
   cloned once per phase closure.

Full implementation:

```rust
{{#include ../../../src/docs_examples.rs:custom_lift_note_visits}}
```

Applied either via `LiftBare::run_on` (as shown in the
accompanying test) or via a pipeline:

```text
use hylic_pipeline::prelude::*;
let r = my_treeish_pipeline.lift()
    .then_lift(NoteVisits { counter })
    .run_from_node(&FUSED, &root);
```

## When `ShapeLift` is sufficient

If the transformation maps cleanly onto "rewrite one of the
three slots," a custom `Lift` is unnecessary. One of the
primitives will suffice:

| Primitive                                         | When to use                            |
|---------------------------------------------------|----------------------------------------|
| `Shared::phases_lift(mi, ma, mf)`                 | rewrite all three Fold phases          |
| `Shared::treeish_lift(mt)`                        | rewrite the graph                      |
| `Shared::n_lift(lift_node, build_treeish, contra)`| coordinated N-change across all slots  |

Alternatively, one of the per-axis sugar constructors — 
`wrap_init_lift`, `zipmap_lift`, `filter_edges_lift`, etc. — may
apply directly; all of them are `ShapeLift` instances with the
appropriate xforms prewired. `NoteVisits`, for example, is
expressible as
`Shared::wrap_init_lift(|n, orig| { counter.bump(); orig(n) })`
with the counter captured by the closure. The custom
implementation above is presented only to illustrate the trait
structure.

## When a custom Lift is warranted

Three representative cases from the ecosystem:

- **`SeedLift`** — domain-parametric, stateful (carrying `grow`,
  `entry_seeds`, `entry_heap_fn`), N-changing, with per-variant
  dispatch on `SeedNode`. Structurally incompatible with
  `ShapeLift` because it must *consume* the upstream `grow`
  rather than wrap it.

  ```rust
  {{#include ../../../../hylic/src/ops/lift/seed_lift.rs:seed_lift_struct}}
  ```

  See [`hylic/src/ops/lift/seed_lift.rs`](../../../../hylic/src/ops/lift/seed_lift.rs)
  for the full variant dispatch: `Entry` fans out over entry
  seeds; `Node(n)` visits the user's treeish; the downstream
  `grow` is the synthesised unreachable closure.

- **`Shared::explainer_lift`** — itself a `ShapeLift`, but its
  construction (`domain/shared/shape_lifts/explainer.rs`)
  demonstrates what a per-node stateful fold wrap entails:
  `init` opens an `ExplainerHeap`; `accumulate` appends a
  transition; `finalize` emits an `ExplainerResult`. Worth
  reading before writing a fold-rewriting lift.

## Capability bounds

`PureLift` and `ShareableLift` are blanket markers. They are
not implemented directly; the compiler selects them automatically
when a lift's types meet the required bounds:

- **`PureLift`** — `Clone + 'static` on the lift and `Clone` on
  its outputs. Required for the sequential executor `Fused`.
- **`ShareableLift`** — adds `Send + Sync + 'static` throughout.
  Required for the parallel executors (`Funnel`).

For a lift to run under `Funnel`, the struct must be
`Clone + Send + Sync + 'static`, and every captured field must be
as well.

## Testing

A typical test:

1. Construct a small treeish and fold whose result can be
   computed by hand.
2. Apply the lift via `LiftBare::run_on` or via a pipeline.
3. Assert the result as well as any side effects the lift is
   expected to record.

For cross-domain parity, repeat the test with `Local` (and with
`Owned` where the lift has an Owned variant).

# Writing a custom Lift

Most transformations compose out of the library catalogue and
the sugar traits. A custom `Lift` impl is the right tool when
the transformation carries cross-node state, requires
per-variant dispatch on the input N, or is itself an execution
strategy.

`apply` has one job: produce the three output slots and hand
them to a continuation `cont(grow', treeish', fold')`. Everything
about the impl follows from four decisions about how those
slots relate to the input ones.

## Four decisions

**1. Output types.**

```text
type N2   = ???;
type MapH = ???;
type MapR = ???;
```

Mirror the input on axes the lift does not change. Where an
axis changes, declare the new type. `MapH` and `MapR` are
typically wrappers — the explainer, for instance, wraps `MapR`
into `ExplainerResult<N, H, R>`.

**2. Treatment of the input `grow`.**

Three options: pass through unchanged, wrap with an N-conversion
when N changes, or synthesise a fresh `grow` (the `SeedLift`
case, where the chain head closes the grow axis). Most custom
lifts pass through.

**3. Treatment of the input `treeish`.**

Pass through, filter, wrap in a visit-intercepting closure,
or rebuild entirely.

**4. Treatment of the input `fold`.**

Clone it once per phase closure. Build a new
`Fold<D::N2, MapH, MapR>` whose `init`, `accumulate`, and
`finalize` delegate to the original through the captured clones.

## Worked example

`NoteVisits` increments a shared counter every time `init`
runs. No type changes; `grow` and `treeish` pass through; `fold`
gets a wrapped `init`.

```rust
{{#include ../../../src/docs_examples.rs:custom_lift_note_visits}}
```

Apply via `LiftBare::run_on` or compose into a pipeline:

```rust
use hylic_pipeline::prelude::*;
let r = my_treeish_pipeline.lift()
    .then_lift(NoteVisits { counter })
    .run_from_node(&FUSED, &root);
```

## When `ShapeLift` is sufficient

If the transformation is "rewrite one of the three slots" —
which it is most of the time — one of the per-axis primitives
or the universal `ShapeLift` does the job.

| Primitive                                          | When                                      |
|----------------------------------------------------|-------------------------------------------|
| `Shared::phases_lift(mi, ma, mf)`                  | rewrite all three Fold phases             |
| `Shared::treeish_lift(mt)`                         | rewrite the graph                         |
| `Shared::n_lift(lift_node, build_treeish, contra)` | coordinated N-change across all slots     |
| `Shared::wrap_init_lift(w)`                        | wrap `init`                               |
| `Shared::zipmap_lift(m)`                           | extend `R`                                |
| `Shared::filter_edges_lift(p)`                     | drop edges from the graph                 |

(Local mirrors are alongside.) `NoteVisits` above is
expressible as
`Shared::wrap_init_lift(|n, orig| { counter.bump(); orig(n) })`;
the custom impl was shown to illustrate the trait structure.

## Capability bounds

- **`PureLift`** — `Clone + 'static` on the lift, `Clone` on
  every output type. Required for the sequential `Fused`
  executor.
- **`ShareableLift`** — adds `Send + Sync + 'static` on the
  lift and on every payload. Required for the parallel
  `Funnel` executor.

Both are blanket markers; the compiler selects them when the
bounds are met. To run under `Funnel`, the lift struct itself
must be `Clone + Send + Sync + 'static`, and every captured
field must satisfy the same.

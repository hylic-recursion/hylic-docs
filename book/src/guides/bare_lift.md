# Bare lift application

You don't have to use the pipeline crate to benefit from lifts.
Any `Lift` implementation can be applied directly to a bare
`(treeish, fold)` pair via the `LiftBare` blanket trait.

## The trait

```rust
{{#include ../../../../hylic/src/ops/lift/bare.rs:lift_bare_trait}}
```

Two methods:

- **`apply_bare(treeish, fold)`** — returns the transformed
  `(treeish', fold')` pair. You take it from here; run it via
  any executor.
- **`run_on(exec, treeish, fold, root)`** — apply + run. Returns
  the lift's `MapR`.

## Example

```rust
{{#include ../../../src/docs_examples.rs:bare_lift_wrap_init}}
```

## When to pick bare over pipeline

- **A single lift, applied once** — pipeline machinery is dead weight.
- **A library on top of hylic** that wants a thin dependency:
  `hylic` alone (no `hylic-pipeline`) is enough.
- **Benchmarking parallel lifts.** `ParLazy` and `ParEager` (in
  `hylic-parallel-lifts`) are `Lift` impls; `run_on` measures
  them without the pipeline in the way.

## Compose without a pipeline

`ComposedLift::compose` takes two `Lift` atoms:

```rust
{{#include ../../../src/docs_examples.rs:bare_lift_composed}}
```

Stage-2 `.then_lift(...)` calls the same primitive.

## The panic-grow

`Lift::apply` takes `(grow, treeish, fold)`; the bare path has no
grow (you start from `&root`). `LiftBare::apply_bare` synthesises
one:

```text
let panic_grow = <D as Domain<N>>::make_grow::<(), N>(|_: &()| {
    unreachable!("LiftBare::apply_bare synthesises a panic-grow; no Lift impl invokes grow at runtime")
});
self.apply::<(), _>(panic_grow, treeish, fold, |_g, t, f| (t, f))
```

No library `Lift` impl reads `grow` at runtime (only `SeedLift`
consumes it, and `SeedLift` doesn't run under `apply_bare`). A
custom Lift that did read grow would panic here instead of
computing a wrong result silently.

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
- **`run_on(exec, treeish, fold, root)`** — applies the lift
  AND runs it in one step. Returns the lift's `MapR`.

## Example

```rust
use hylic::prelude::*;

let treeish = treeish(|n: &u64| if *n > 0 { vec![*n - 1] } else { vec![] });
let fld     = fold(|n: &u64| *n, |h: &mut u64, c: &u64| *h += c, |h: &u64| *h);

// Direct: a single lift applied to a bare pair.
let trace_lift = Shared::explainer_lift::<u64, u64, u64>();
let r = trace_lift.run_on(&FUSED, treeish.clone(), fld.clone(), &5u64);
println!("result = {}", r.orig_result);
```

## When to pick bare over pipeline

- **You don't need chaining.** A single lift applied one-shot —
  pipeline machinery is overhead.
- **You're building a library on top of hylic** and want the
  thinnest possible dependency. `hylic` alone (no
  `hylic-pipeline`) suffices.
- **You're benchmarking parallel lifts.** `ParLazy` and
  `ParEager` (in the `hylic-parallel-lifts` crate) are `Lift`
  impls; `run_on` is the canonical benchmark entry point —
  no pipeline overhead in the measurement.

## Compose first, run later

You can still compose multiple lifts without a pipeline — just
use `ComposedLift::compose`:

```rust
use hylic::ops::ComposedLift;

let l1 = Shared::wrap_init_lift::<u64, u64, u64, _>(|n, orig| orig(n) + 1);
let l2 = Shared::zipmap_lift::<u64, u64, u64, bool, _>(|r: &u64| *r > 5);
let composed = ComposedLift::compose(l1, l2);

let (r, flag) = composed.run_on(&FUSED, treeish, fld, &5u64);
```

This is what Stage-2 `.then_lift(...)` does under the hood —
bare usage just exposes the underlying atom.

## The "panic-grow" trick

`Lift::apply` takes three inputs: grow, treeish, fold. But the
bare path has no grow (you start from `&root`, not from a seed).
`LiftBare::apply_bare` synthesises a panic-grow:

```rust
let panic_grow = <D as Domain<N>>::make_grow::<(), N>(|_: &()| {
    unreachable!("LiftBare::apply_bare synthesises a panic-grow; no Lift impl invokes grow at runtime")
});
self.apply::<(), _>(panic_grow, treeish, fold, |_g, t, f| (t, f))
```

Why does this work? Because no library `Lift` impl actually
*reads* the grow argument; they pass it through (grow is only
consumed by `SeedLift`, which is NOT used in bare-execution paths).
The panic is there for correctness: if some custom Lift broke
this invariant, you'd find out at run time, not silently get a
wrong answer.

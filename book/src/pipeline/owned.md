# One-shot — OwnedPipeline

`OwnedPipeline` is the out-of-band variant: one-shot, `Box`-based
storage, no `Clone`, no Stage-2 sugar surface. Use it when:

- You run exactly once.
- You want minimal allocation / closure overhead.
- You don't need to transform the pipeline after construction.

```rust
{{#include ../../../../hylic-pipeline/src/owned/mod.rs:owned_pipeline_struct}}
```

## Why no Stage-2

`Owned` deliberately isn't `ShapeCapable`. Since the pipeline is
consumed on first (and only) use, there's nothing to lift —
transforms would have to consume and rebuild the whole thing,
which defeats the purpose of this domain.

The primitive you do get: `new(treeish, fold).run_from_node_once(&exec, &root)`.

## Example

```rust
use hylic_pipeline::{OwnedPipeline, PipelineExecOnce};
use hylic::domain::owned as odom;

let graph   = odom::edgy::treeish(|n: &u64| if *n > 0 { vec![*n - 1] } else { vec![] });
let fld     = odom::fold(|n: &u64| *n, |h: &mut u64, c: &u64| *h += c, |h: &u64| *h);

let r: u64 = OwnedPipeline::new(graph, fld)
    .run_from_node_once(&odom::FUSED, &5u64);
//    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//    consumes self — the pipeline is gone after this call.
```

## Relation to bare usage

`OwnedPipeline::run_from_node_once(&exec, &root)` is equivalent
to running the (treeish, fold) pair through the executor
directly — it's a tiny convenience that packages the two slots
and provides one consistent method name.

If you have a lift you want to apply once, use
[`LiftBare::run_on`](../concepts/lifts.md#applying-a-lift-without-a-pipeline)
with Shared or Local; there's no Owned equivalent because Owned
lifts don't exist.

## When not to use this

- Any time you want chainable transforms — use Shared or Local.
- Any time you run the same fold more than once — use Shared
  (Clone lets you reuse).
- Any time you need parallelism — Owned isn't `Send + Sync`.

In practice, Owned is the right choice for scripts, one-off
tools, and minimal-overhead benchmarks.

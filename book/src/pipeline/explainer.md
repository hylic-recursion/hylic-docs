# Case study — Explainer

`explainer_lift` is a ShapeLift constructor that wraps a fold
with per-node trace recording. It's a useful case study because
it changes **H and R** (not N), composes as a post-lift, and
produces a result type that lets callers inspect the full
computation tree.

## What it does

```rust
{{#include ../../../../hylic/src/domain/shared/shape_lifts/explainer.rs:explainer_lift_ctor}}
```

The lift wraps:

- **H** becomes `ExplainerHeap<N, H, ExplainerResult<N, H, R>>`:
  the original H plus a vector of per-child transitions recorded
  during accumulate.
- **R** becomes `ExplainerResult<N, H, R>`: the original result
  plus the full heap (so callers can walk the trace tree).

Every node's finalize produces both the original R and the
recorded history.

## Usage

Via the sugar method `.explain()` on any Stage-2 pipeline (or
Stage-1 via auto-lift):

```rust
{{#include ../../../src/docs_examples.rs:explainer_usage}}
```

The return type is `ExplainerResult<N, H, R>`. Access
`.orig_result` for the original computation's output:

```rust
let trace: ExplainerResult<Node, u64, u64> = pipeline.explain().run_from_node(&FUSED, &root);
assert_eq!(trace.orig_result, 42);  // the value the original fold would have produced
// trace.heap.transitions gives the per-child accumulation history
```

## Composing with other lifts

Because `explain()` is just a `then_lift(Shared::explainer_lift())`,
it composes:

```rust
let r = pipeline
    .wrap_init(|n, orig| orig(n) * 2)   // first lift
    .explain()                           // records the wrap_init results
    .zipmap(|r| r.orig_result > 100);    // inspect .orig_result
```

Order matters: lifts run bottom-up (the first `.wrap_init` runs
innermost; `.explain` sees its results; `.zipmap` sees the
`ExplainerResult`).

## Streaming variant

`Shared::explainer_describe_lift(fmt, emit)` is a variant that
emits formatted trace lines per node via a callback but keeps
R transparent (`MapR = R`). Use it when you want live per-node
trace output without changing your pipeline's result type:

```rust
use hylic::prelude::*;
let _ = Shared::explainer_describe_lift::<Node, u64, u64, _, _>(
    trace_fold_compact::<Node, u64, u64>,
    |line: &str| eprintln!("[trace] {line}"),
);
```

Local mirror of `explainer_describe_lift` is deferred (blocked
on `Send+Sync` in the formatter); the whole-trace `explainer_lift`
is available for Local.

## What this shows about the lift model

1. **H and R can change independently.** Their types are bundled
   by a single ShapeLift, but the lift is free to pick any shape
   it needs.
2. **Downstream lifts see the new shape.** If you chain
   `.explain().zipmap(…)`, the `zipmap` closure operates on
   `ExplainerResult<N, H, R>`, not the original R.
3. **The type system carries the shape.** Your `.run_from_node(...)`
   return type reflects every lift in the chain — you can see at
   compile time what result type to expect.

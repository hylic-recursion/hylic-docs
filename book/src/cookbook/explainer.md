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

Via the sugar method `.explain()` on any Stage-2 pipeline — a
treeish-rooted `Stage2Pipeline`, a seed-rooted `Stage2Pipeline`,
or a `TreeishPipeline` via auto-lift. A `SeedPipeline` requires
an explicit `.lift()` first:

```rust
{{#include ../../../src/docs_examples.rs:explainer_usage}}
```

The return type is `ExplainerResult<N', H, R>` where `N'` is the
chain's current node type — `N` on a treeish-rooted chain, but
`SeedNode<N>` on a seed-rooted chain (since the seed chain's
node type is `SeedNode<N>` from `.lift()` onward). Access
`.orig_result` for the original computation's output:

```rust
{{#include ../../../src/docs_examples.rs:explainer_orig_result}}
```

### Sealed view on the seed path

For an N-typed view of the trace that hides `SeedNode` entirely,
project via the standard `From` conversion:

```text
use hylic::prelude::SeedExplainerResult;

let raw: ExplainerResult<SeedNode<N>, H, R> =
    pipeline.lift().explain().run_from_slice(&FUSED, &seeds, h0);
let sealed: SeedExplainerResult<N, H, R> = raw.into();

// sealed.entry_initial_heap, entry_working_heap, orig_result — EntryRoot row promoted out
// sealed.roots: Vec<ExplainerResult<N, H, R>>                — per-seed subtrees
```

Use `raw` when you need to keep composing lifts on top of
`.explain()` (the chain type is what matters); use `sealed` when
you want an N-typed view for formatting or assertions — the
library's invariant guarantees every below-root node is a
`Node(n)`, so the unwrap is total.

## Composing with other lifts

Because `explain()` is just a `then_lift(Shared::explainer_lift())`,
it composes:

```text
let r = pipeline
    .wrap_init(|n, orig| orig(n) * 2)   // first lift
    .explain()                           // records the wrap_init results
    .zipmap(|r| r.orig_result > 100);    // inspect .orig_result
```

Order matters: lifts run bottom-up (the first `.wrap_init` runs
innermost; `.explain` sees its results; `.zipmap` sees the
`ExplainerResult`).

## Streaming variant

`Shared::explainer_describe_lift(fmt, emit)` emits formatted
trace lines per node via a callback and leaves `MapR = R`
unchanged:

```text
use hylic::prelude::*;
let _ = Shared::explainer_describe_lift::<Node, u64, u64, _, _>(
    trace_fold_compact::<Node, u64, u64>,
    |line: &str| eprintln!("[trace] {line}"),
);
```

Local mirror deferred (blocked on `Send+Sync` in the formatter);
`explainer_lift` is available for Local.

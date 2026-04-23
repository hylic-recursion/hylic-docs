# Writing a custom Lift

The library's shape-lift catalogue covers the common cases. When
you need a lift with state, cross-node coordination, or a type
transformation the catalogue doesn't support, you write your own
`Lift` impl.

## What you implement

`Lift<D, N, H, R>` declares three output types and one CPS method:

```rust
{{#include ../../../../hylic/src/ops/lift/core.rs:lift_trait}}
```

Your `apply` is handed the three input slots and a continuation.
You build the three output slots (whatever types your lift
produces) and pass them to `cont`.

## Minimal template (Shared, no type change)

A no-op lift for Shared that just wraps the fold's init:

```rust
{{#include ../../../src/docs_examples.rs:custom_lift_note_visits}}
```

This `NoteVisits` lift counts init calls into a shared
`Arc<Mutex<u64>>`. The pattern that makes it work: clone the
input `fold` three times (one per phase) before constructing
the output fold, so each phase closure owns an independent
handle. The `cont` call at the end hands the unchanged `grow`
and `treeish` plus the wrapped fold to the downstream chain.

Once defined, use it via `LiftBare::run_on` (shown inside the
test) or via a pipeline:

```text
use hylic_pipeline::prelude::*;
let r = my_treeish_pipeline.lift().then_lift(NoteVisits { counter })
    .run_from_node(&FUSED, &root);
```

## When `ShapeLift` beats a hand-rolled `Lift`

If your lift can be described as **"rewrite one or more of the
three xforms (grow, treeish, fold)"**, prefer building a
`ShapeLift` via the existing primitives:

- `Shared::phases_lift(mi, ma, mf)` — rewrite all three fold phases.
- `Shared::treeish_lift(mt)` — rewrite the graph.
- `Shared::n_lift(lift_node, build_treeish, fold_contra)` — change N
  across all three slots in one coordinated move.

You go custom when:

- You need **cross-axis state** (e.g. a memoisation cache threaded
  through multiple nodes' folds).
- You need **different type transformations on different
  variants** (e.g. `SeedLift`'s `LiftedNode::Entry` vs `Node(n)`
  dispatch).
- You're implementing a **domain-specific execution strategy** (e.g.
  `ParLazy` and `ParEager` in the `hylic-parallel-lifts` crate —
  they *are* `Lift<Shared, N, H, R>` impls whose `apply` produces
  a fold that schedules work onto a thread pool).

## SeedLift as a reference

The library's `SeedLift` is the clearest non-trivial example.
It's Shared-pinned, has state (grow + entry_seeds +
entry_heap_fn), and changes N to `LiftedNode<N>`:

```rust
{{#include ../../../../hylic/src/ops/lift/seed_lift.rs:seed_lift_struct}}
```

Look at [`hylic/src/ops/lift/seed_lift.rs`](../../../../hylic/src/ops/lift/seed_lift.rs)
end-to-end for the full picture: how it constructs a
`Treeish<LiftedNode<N>>` with per-variant dispatch, how it
wraps the fold to handle Entry's synthetic root heap, and how it
produces an "unreachable" downstream grow (no further lift can
observe it, because the chain is closed).

## Testing your lift

Write a `#[test]` that:

1. Constructs a simple treeish + fold.
2. Computes the expected result without your lift.
3. Applies your lift via `LiftBare::run_on` or a pipeline.
4. Asserts the result (and any side effects your lift records).

For cross-domain parity (if your lift has Local / Owned
variants), write the same test twice with the respective domain.

## Capability bounds

If your lift needs to run under parallel executors (Funnel),
make it `Clone + Send + Sync + 'static` with
`Send + Sync + 'static` on every output type. The blanket
`ShareableLift` marker will pick it up automatically.

Sequential executors (`Fused`) only need `PureLift`, which
everything Clone + 'static satisfies automatically.

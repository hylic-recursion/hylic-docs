# Implementing a custom lift

A lift transforms both fold and treeish into a different type domain.
The `LiftOps` trait provides three methods — `lift_treeish`,
`lift_fold`, `lift_root`. This page walks through the implementation
pattern using SeedLift (the lift inside
[`SeedPipeline`](./seed_pipeline.md)) and the Explainer as examples.

## Step 1: define the lifted heap

A lift transforms the fold's heap type `H` into something richer.
The lifted heap dispatches between the original fold's logic and the
lift's own logic:

```rust
{{#include ../../../../hylic/src/cata/seed_lift.rs:seed_heap}}
```

SeedLift's heap has two variants: `Node(H)` delegates to the
original fold, `Relay(Option<R>)` stores a single child's result
for pass-through. The Explainer uses the same pattern — its
`ExplainerHeap<N, H, R>` wraps the original `H` and adds trace
fields. In both cases, the lifted heap is a GAT:
`type LiftedH<H: Clone + 'static> = SeedHeap<H, R>`.

## Step 2: implement `lift_treeish`

SeedLift changes the node type (`N → Either<Seed, N>`) using the
same `.map` + `.contramap_or` combinators that
[`SeedPipeline` uses](./seed_pipeline.md#how-it-works-internally)
to construct its treeish — widen the edge type, then close the node
side:

```
treeish.map(Right)                  N → Either<Seed, N>
       .contramap_or(dispatch)      Either<Seed, N> → delegate or produce
```

After one `Left → Right` transition, the original treeish drives all
further traversal. The lift converges to the original computation
within one step.

Not every lift changes the node type. The Explainer's `lift_treeish`
returns the input treeish unchanged — it only transforms the fold.

## Step 3: implement `lift_fold`

The lifted fold dispatches per phase (init, accumulate, finalize)
based on the lifted heap variant:

- **init**: `Right(node)` → `Node(f.init(node))`.
  `Left(seed)` → `Relay(None)`.
- **accumulate**: `Node(h)` → `f.accumulate(h, result)`.
  `Relay(slot)` → store the child result.
- **finalize**: `Node(h)` → `f.finalize(h)`.
  `Relay(Some(r))` → return `r` (transparent pass-through).

The Explainer follows the same structure: init wraps the original
heap in a trace container, accumulate records the transition then
delegates, finalize produces both the original result and the trace.

## Step 4: `lift_root`

`lift_root` converts the user's root into the lifted node type.
SeedLift wraps it as `Right(root.clone())` — the root is already a
resolved node. The Explainer clones the root unchanged.

## The trait

```rust
{{#include ../../../../hylic/src/ops/lift.rs:liftops_trait}}
```

`run_lifted` applies the three methods, runs the lifted computation
through an executor, and returns `LiftedR<H>`. The caller extracts
the original result as appropriate — `ExplainerResult::orig_result`
for the Explainer, or `R` directly when `LiftedR<H> = R` (SeedLift).

## From lift to user-facing abstraction

A raw lift requires the user to manage `Either<Seed, N>` types,
construct the lifted treeish and fold, and run the executor manually.
[`SeedPipeline`](./seed_pipeline.md) wraps SeedLift into a
user-facing API that hides these internals: the user provides `grow`,
`seeds_from_node`, and a fold over `N`; the pipeline handles the
lift, the treeish composition, and the entry transition. The
`Either` type never appears in the user's code.

This pattern — implement a `LiftOps` for the internal mechanics,
then wrap it in a pipeline or adapter that presents a clean API — is
how hylic separates the algebra morphism (the lift) from the
ergonomics (the wrapper).

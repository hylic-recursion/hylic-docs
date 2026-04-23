# Implementing a custom lift

A lift transforms both fold and treeish into a different type domain.
The `Lift` trait provides three methods — `lift_treeish`,
`lift_fold`, `lift_root`. This page walks through the implementation
pattern using SeedLift (the lift inside
[`SeedPipeline`](./seed_pipeline.md)) and the Explainer as examples.

## Step 1: define the lifted heap

A lift transforms the fold's heap type `H` into something richer.
The lifted heap dispatches between the original fold's logic and the
lift's own logic:

```rust
{{#include ../../../../hylic/src/cata/seed_lift/types.rs:seed_heap}}
```

SeedLift's heap has two variants: `Active(H)` delegates to the
original fold, `Relay(Option<R>)` stores a single child's result
for pass-through. The Explainer uses the same pattern — its
`ExplainerHeap<N, H, R>` wraps the original `H` and adds trace
fields. In both cases, the lifted heap is a GAT:
`type MapH<H, R> = LiftedHeap<H, R>`.

## Step 2: implement `lift_treeish`

SeedLift changes the node type (`N → LiftedNode<Seed, N>`) by
constructing a `Treeish<LiftedNode<Seed, N>>` that dispatches per
variant — see
[`SeedPipeline` internals](./seed_pipeline.md#how-it-works-internally):

```
Node(n)  → visit original treeish, wrap children as Node
Seed(s)  → produce one child: Node(grow(s))
Entry    → children from entry_seeds, wrapped as Seed
```

After the `Seed → Node` transition, the original treeish drives all
further traversal. The lift converges to the original computation
within one step.

Not every lift changes the node type. The Explainer's `lift_treeish`
returns the input treeish unchanged — it only transforms the fold.

## Step 3: implement `lift_fold`

The lifted fold dispatches per phase (init, accumulate, finalize)
based on the lifted heap variant:

- **init**: `Node(n)` → `Active(f.init(n))`.
  `Seed(_)` → `Relay(None)`.
  `Entry` → `Active(f.init(...))` with the entry heap.
- **accumulate**: `Active(h)` → `f.accumulate(h, result)`.
  `Relay(slot)` → store the child result.
- **finalize**: `Active(h)` → `f.finalize(h)`.
  `Relay(Some(r))` → return `r` (transparent pass-through).

The Explainer follows the same structure: init wraps the original
heap in a trace container, accumulate records the transition then
delegates, finalize produces both the original result and the trace.

## Step 4: `lift_root`

`lift_root` converts the user's root into the lifted node type.
SeedLift wraps it as `Node(root.clone())` — the root is already a
resolved node. The Explainer clones the root unchanged.

## The trait

```rust
{{#include ../../../../hylic/src/ops/lift/core.rs:lift_trait}}
```

`Lift<N, N2>` is a bifunctor on the `(H, R)` pair — both `MapH`
and `MapR` are GATs parameterized by `(H, R)`. This enables blanket
composition via `ComposedLift` without boilerplate. `run_lifted`
applies the three methods, runs the lifted computation through an
executor, and returns `MapR<H, R>`.

## From lift to user-facing abstraction

A raw lift requires the user to manage `LiftedNode<Seed, N>` types,
construct the lifted treeish and fold, and run the executor manually.
[`SeedPipeline`](./seed_pipeline.md) wraps SeedLift into a
user-facing API that hides these internals: the user provides `grow`,
`seeds_from_node`, and a fold over `N`; the pipeline handles the
lift, the treeish composition, and the entry transition. The
`LiftedNode` type never appears in the user's code.

This pattern — implement a `Lift` for the internal mechanics,
then wrap it in a pipeline or adapter that presents a clean API — is
how hylic separates the algebra morphism (the lift) from the
ergonomics (the wrapper).

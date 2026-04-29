# `SeedNode<N>` — the seed-rooted row type

A `SeedPipeline` carries no concept of "the chain root" until execution:
at `.run(...)` time (or `.lift().run(...)` time, when Stage-2 sugars are
chained), the library synthesises one. The cost is a single extra
inhabitant in the chain's node type — the synthetic `EntryRoot`.

```rust
{{#include ../../../../hylic/src/ops/lift/seed_node.rs:seed_node_enum}}
```

`SeedNode<N>` is the chain-tip node type once `SeedLift` has fired:

- **`Node(N)`** — a real grown node from the user's seed graph.
- **`EntryRoot`** — the synthetic forest root. Its children are the
  user-supplied entry seeds, grown into nodes.

Sealed at the variant level. Reading the row from user code goes through
[`is_entry_root()`](../../../../hylic/src/ops/lift/seed_node.rs),
[`as_node()`](../../../../hylic/src/ops/lift/seed_node.rs), and
[`map_node`](../../../../hylic/src/ops/lift/seed_node.rs); pattern-matching
the variants from outside the crate is not allowed. Stage-2 sugars hide the
row entirely — every closure types at `&N`, never `&SeedNode<N>`.

The seal is motivated by the next two points: where `SeedNode<N>`
nevertheless surfaces, and how to project it back out.

## Where the row surfaces

A handful of lifts propagate the chain's N into their output type. The
explainer is the canonical example:

```rust,ignore
let raw: ExplainerResult<SeedNode<N>, H, R> = pipeline
    .lift()
    .explain()
    .run_from_slice(&exec, &seeds, h0);
```

`ExplainerResult`'s first parameter is the per-node `heap.node` — and on a
seed-rooted chain that's `SeedNode<N>`. The same pattern applies to any
custom `Lift` whose output type mentions its input's `N`.

For uses that walk the trace (formatting, post-fact analysis), the sealed
view is awkward. Project to an N-typed view via
[`SeedExplainerResult::from`](../../../../hylic/src/prelude/explainer.rs):

```rust,ignore
let sealed: SeedExplainerResult<N, H, R> = raw.into();
// sealed.entry_initial_heap, sealed.entry_working_heap, sealed.orig_result
//   — the EntryRoot row, promoted out of the tree as fields
// sealed.roots: Vec<ExplainerResult<N, H, R>>
//   — per-seed subtrees, every node now plain N
```

The conversion is total: every node below the EntryRoot row is unwrapped.
`SeedNode<N>` no longer appears in the user-visible shape.

## Why the chain has to operate on `SeedNode<N>`

The executor's `run` takes a single root — `run(fold, treeish, &N) → R`.
But a `SeedPipeline` admits a *forest* of entry seeds. The library's
choice (over a native-forest executor, see
[design notes](../design/pipeline_transformability.md)) is to invent a
synthetic single root at the chain head: `SeedNode::EntryRoot`. Its
children are `grow(seed)` for each entry seed.

```text
        EntryRoot
        ├── Node(grow(seed_0))
        ├── Node(grow(seed_1))
        └── …

Each Node(n) below has the user's seeds_from_node + grow as its
own children-producing function.
```

Because `SeedLift` is composed at the **head** of the chain (run-time, not
storage-time), every subsequent stored lift in `L` sees its input as
`SeedNode<N>`. The user's Stage-2 sugars dispatch closures back down to
`&N` via [Wrap dispatch](./wrap_dispatch.md); the row only escapes user
view when a chain-tip type carries the chain's N (the explainer case
above).

## Quick reference

| Operation                              | Returns                                  |
|----------------------------------------|------------------------------------------|
| `sn.is_entry_root()`                   | `bool`                                   |
| `sn.as_node()`                         | `Option<&N>`                             |
| `sn.map_node(f)`                       | `SeedNode<M>` (Node mapped, EntryRoot preserved) |
| `raw_explained.into()` → `SeedExplainerResult<N, H, R>` | EntryRoot promoted to fields; subtrees projected to plain N |

The rule of thumb: **inside Stage-2 sugar bodies, you never see
`SeedNode<N>`. At the chain tip, you see it iff the lift's output type
mentions the chain's N.** The latter is rare (explainer is the main
case); the former never happens.

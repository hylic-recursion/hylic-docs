# Glossary

One-line definitions of the core terms, with pointers to where
they're developed in depth. Link to this page from anywhere a
term appears without its definition.

### Fold

The algebra over a recursion — a triple of closures `init: &N → H`,
`accumulate: &mut H, &R`, `finalize: &H → R`. Given an input node
`N`, a fold says how to produce a per-node scratch state, how to
fold each child's result into it, and how to close it out into a
final `R`.  See [Fold guide](./guides/fold.md).

### Graph / `Treeish<N>`

A function from a node to its children. The type is
`Treeish<N>`; "graph" is the informal name for the concept. A
`Treeish` is how the recursion finds the next level; the executor
and fold never see the tree structure directly, only what the
`Treeish` yields. See [Graph guide](./guides/treeish.md).

### Heap (`H`)

The per-node working state inside a fold. Produced by `init`,
mutated by `accumulate` as each child's `R` arrives, consumed by
`finalize`. Not shared across nodes; each node gets its own. Also
written as the type variable `H` in Fold signatures.

### `R` (result)

The type returned from `finalize` at each node, and the type that
flows upward into the parent's `accumulate`. At the root, the
executor hands back an `R`.

### Domain (`Shared` / `Local` / `Owned`)

How hylic stores closures inside folds, graphs, and grow
functions. `Shared` uses `Arc<dyn Fn + Send + Sync>` (cheap clone,
parallel-safe); `Local` uses `Rc<dyn Fn>` (cheap clone,
single-thread, non-`Send` captures); `Owned` uses `Box<dyn Fn>`
(one-shot, consumed on use). See
[The three domains](./concepts/domains.md).

### Executor (`Fused` / `Funnel`)

The runtime that drives the recursion. `Fused` is a direct
sequential callback recursion — one thread, no work queue.
`Funnel` is a parallel work-stealing engine running over a
compile-time policy (queue topology, accumulation strategy, wake
policy). Both implement the `Executor<N, R, D, G>` trait. See
[Choosing an executor](./guides/execution.md).

### Lift

A transformation that rewrites a `(grow, treeish, fold)` triple
into another one with possibly different types. Implemented by
the `Lift` trait; the library ships four atoms (`IdentityLift`,
`ComposedLift`, `ShapeLift`, `SeedLift`). See
[Lifts](./concepts/lifts.md).

### `ComposedLift<L1, L2>`

The binary composition atom — two lifts chained so `L2`'s inputs
equal `L1`'s outputs. Every multi-sugar call builds a
right-associated `ComposedLift` tree so the compiler can verify
every junction at build time.

### `ShapeLift`

The universal "rewrite one or more fold phases" lift used by
most Stage-2 sugars (`wrap_init`, `zipmap`, `filter_edges`, …).

### `SeedLift`

The finishing lift that closes a `SeedPipeline`'s grow axis.
Domain-generic (`SeedLift<D, N, Seed, H>`; impls for `Shared` and
`Local`). Not something user code constructs directly —
`LiftedSeedPipeline::run` assembles it at call time from
`grow` + user-supplied `root_seeds` + `entry_heap` and composes
it as the first lift of the run-time chain.

### `LiftedNode<N>`

Sealed row type with two library-internal variants: the
synthetic `Entry` (a seed-closed chain's root row) and a
resolved `Node(N)`. User code inspects via `is_entry`,
`as_node`, `map_node`.

### `SeedExplainerResult<N, H, R>`

N-typed projection of a seed-closed explainer result. The Entry
row is promoted into top-level fields (`entry_initial_heap`,
`entry_working_heap`, `orig_result`); each root subtree becomes
an `ExplainerResult<N, H, R>` — no `LiftedNode<N>` appears in
the user-visible shape. Obtained via
`SeedExplainerResult::from_lifted(raw)`.

### Pipeline

A typestate-chained builder over lifts. `SeedPipeline` and
`TreeishPipeline` are Stage 1 (base slots); `LiftedPipeline` is
Stage 2 (base + lift chain); `OwnedPipeline` is a one-shot
variant. Every pipeline ultimately resolves to a `(treeish, fold)`
pair handed to an executor. See [Pipelines](./pipeline/overview.md).

### Sugar

A pipeline method that delegates to `.then_lift(...)` with a
library lift — `wrap_init`, `zipmap`, `filter_edges`, `explain`,
etc. Sugars on `TreeishPipeline` and `LiftedPipeline` live on a
blanket trait (`LiftedSugarsShared` / `LiftedSugarsLocal`);
`LiftedSeedPipeline` exposes an inherent mirror of the same
surface (whose chain is typed at `LiftedNode<N>`, so it cannot
share the trait's parameter shape). See
[Blanket sugar traits](./pipeline/sugars.md).

### CPS (continuation-passing style)

Used in two places with different meanings, both internal
machinery:

- In `Lift::apply`, the trait takes a continuation so a lift can
  transform the triple and then *call through* to the user's
  executor rather than returning a value. This enables composition
  without an intermediate materialisation.
- In the Funnel executor, the recursion is defunctionalised into
  `Cont::Root / Cont::Direct / Cont::Slot` variants so workers run
  a `loop { match cont { … } }` rather than nesting calls. See
  [CPS walk](./funnel/cps_walk.md).

Users of the library don't need to think about CPS to use it;
these sections are optional reading.

### Variance

Whether a type's role allows covariant, contravariant, or
invariant transformation. `N` is covariant in grow, invariant in
graph, contravariant in fold's `init`; `H` and `R` are invariant.
This is why the methods have the names they do
(`map` for covariant, `contramap` for contravariant, `*_bi` for
invariant/bijective). See [Transforms and variance](./concepts/transforms.md).

### Grow

The `Seed → N` closure in a `SeedPipeline` — resolves a reference
into a full node. Only `SeedPipeline` has a grow slot; a
`TreeishPipeline` skips it (nodes are already materialised).

# Lifts — the CPS triple transformer

## The problem

You've built a `Fold<N, H, R>` and a `Graph<N>`. You want to derive
a new fold+graph pair whose types differ from the originals — maybe
R changes, maybe the graph is filtered, maybe nodes get annotated,
maybe the fold gains per-node tracing. The transforms from the
[previous chapter](./transforms.md) operate on one of Fold or
Graph. What you want here is a *composable* object that rewrites
both together and can stack with other such objects.

That object is a **Lift**. It transforms a *triple*, not a pair.

## Why three axes

Besides `Fold<N, H, R>` and `Graph<N>`, there's a third slot:
`Grow<Seed, N>` — a closure that resolves a `Seed` into an `N`.
Most users never build a Grow by hand; [`SeedPipeline`](../pipeline/seed.md)
constructs one from `grow: Seed → N`. But because lifts compose
and some pipelines carry a Grow, the trait has to account for it.

Three slots, not two. The [`Lift`](./lifts.md) trait threads all
three through composition. A lift that doesn't care about Grow
(say, a fold-wrapper) passes it unchanged. A lift that does care
(the N-change lifts; `SeedLift`) rewrites it in concert with the
other slots.

## The trait

```rust
{{#include ../../../../hylic/src/ops/lift/core.rs:lift_trait}}
```

Three associated output types (`N2`, `MapH`, `MapR`) and a single
`apply` method. As a type-level arrow:

```
L : (Grow<Seed, N>, Graph<N>, Fold<N, H, R>)
  → (Grow<Seed, L::N2>, Graph<L::N2>, Fold<L::N2, L::MapH, L::MapR>)
```

## Why CPS

The obvious signature would be to *return* the transformed triple
from `apply`. Try it: the return type is
`(Grow<D, Seed, N2>, Graph<D, N2>, Fold<D, N2, H2, R2>)` where each
of those is a domain-associated GAT and each axis is an associated
type of the lift. After three chained lifts, the return type is
unnameable without a fresh alias per call site.

CPS sidesteps this. The caller hands `apply` a continuation; the
lift fills in its three output types and invokes the continuation
with the transformed triple. The continuation does whatever it
wants — typically calls another lift's `apply` — and the caller
ends up with whatever that continuation returned. Rust's type
inference threads the associated types through end-to-end; nothing
needs a nameable intermediate.

This is why every pipeline's `.run(...)` method ends up being a
single walk down the lift chain via nested `apply` calls, each
closing over the next: the chain is built at the type level, run
once at the value level, and the executor only ever sees the final
`(treeish, fold)` pair.

## Four atoms

Every library lift is one of four things.

**`IdentityLift`** — pass-through. Used as the seed of a lift chain
when a Stage-1 pipeline transitions to Stage 2 via `.lift()`.

```rust
{{#include ../../../../hylic/src/ops/lift/identity.rs:identity_lift}}
```

**`ComposedLift<L1, L2>`** — sequential composition. `L1` runs
first; `L2` takes `L1`'s outputs as its inputs.

```rust
{{#include ../../../../hylic/src/ops/lift/composed.rs:composed_lift}}
```

```dot process
digraph {
    rankdir=LR;
    node [shape=box, style="rounded,filled", fontname="monospace", fontsize=10];
    edge [fontname="sans-serif", fontsize=9];
    in   [label="(N, H, R)", fillcolor="#e8f5e9"];
    mid  [label="(L1::N2, L1::MapH, L1::MapR)", fillcolor="#fff3cd"];
    out  [label="(L2::N2, L2::MapH, L2::MapR)", fillcolor="#e3f2fd"];
    in -> mid [label="L1::apply"];
    mid -> out [label="L2::apply"];
}
```

The type-level bound `L2: Lift<D, L1::N2, L1::MapH, L1::MapR>`
enforces the connection. A mistake here surfaces as a compile
error at the composition site.

**`ShapeLift<D, N, H, R, N2, H2, R2>`** — the universal library
lift. Stores three per-domain xforms (one per slot) and applies
them in sequence.

```rust
{{#include ../../../../hylic/src/ops/lift/shape.rs:shape_lift_struct}}
```

Every concrete library lift is a `ShapeLift` with appropriate
xforms. `wrap_init_lift` only rewrites Fold's init phase;
`filter_edges_lift` only rewrites Graph's visit; `n_lift` rewrites
all three; `explainer_lift` rewrites only Fold (but changes
`MapH` and `MapR` to the explainer's wrapper types).

**`SeedLift<N, Seed, H>`** — a finishing lift that closes a
SeedPipeline by turning the `(grow, seeds_from_node, fold)` triple
into a runnable `(treeish, fold)` pair rooted at an `Entry`
variant. Shared-pinned; used internally by
`PipelineExecSeed::run(...)`.

```rust
{{#include ../../../../hylic/src/ops/lift/seed_lift.rs:seed_lift_struct}}
```

Its N2 is `LiftedNode<N>`:

```rust
{{#include ../../../../hylic/src/ops/lift/lifted_node.rs:lifted_node_enum}}
```

`Entry` is the root fan-out over entry seeds; `Node(N)` is a
resolved node. `SeedLift` builds a `Treeish<LiftedNode<N>>` that
dispatches on variant: Entry visits the entry seeds via grow,
Node visits the user's treeish.

## Bare application

Any `Lift` is usable without a pipeline. `LiftBare` is a blanket
trait:

```rust
{{#include ../../../../hylic/src/ops/lift/bare.rs:lift_bare_trait}}
```

Pair with a `(treeish, fold)` directly:

```rust
{{#include ../../../src/docs_examples.rs:bare_lift_wrap_init}}
```

See [Bare lift application](../guides/bare_lift.md).

## Per-domain capability

Not every domain supports `ShapeLift`. A domain has to declare
what it can store as a per-slot xform:

```rust
{{#include ../../../../hylic/src/ops/lift/capability.rs:shape_capable}}
```

`Shared` and `Local` are `ShapeCapable` — each storage uses its
own pointer type (Arc vs Rc) and closure bounds (`Send + Sync`
vs none). `Owned` is **not** `ShapeCapable`: `Box<dyn Fn>` is not
`Clone`, so xforms can't be applied to produce a new owned fold.
Owned pipelines have no Stage-2 surface.

## Parallel vs sequential

Two blanket markers gate which executors a lift can feed:

- `PureLift<D, N, H, R>` — any `Lift + Clone + 'static` with
  `Clone` outputs. Sufficient for sequential executors (`Fused`).
- `ShareableLift<D, N, H, R>` — adds `Send + Sync` on everything.
  Required for parallel executors (`Funnel`, ParLazy, ParEager).

You don't implement these; the compiler picks them up via blanket
impls in [`ops::lift::capability`](../../../../hylic/src/ops/lift/capability.rs).
If your lift (or your data) doesn't meet the parallel bounds,
calling `.run(&funnel_exec, ...)` is a compile error — there's
no silent fallback.

## Library catalogue

Each `ShapeCapable` domain exposes a set of constructors that
return a `ShapeLift` shaped for the transformation. For `Shared`:

| Constructor                        | What it changes                                     |
|------------------------------------|-----------------------------------------------------|
| `Shared::wrap_init_lift(w)`        | intercept `init` at every node                      |
| `Shared::wrap_accumulate_lift(w)`  | intercept `accumulate`                              |
| `Shared::wrap_finalize_lift(w)`    | intercept `finalize`                                |
| `Shared::zipmap_lift(m)`           | extend R: `R → (R, Extra)`                          |
| `Shared::map_r_bi_lift(fwd, bwd)`  | change R (bijection required; R is invariant)       |
| `Shared::filter_edges_lift(pred)`  | drop edges matching a predicate                     |
| `Shared::wrap_visit_lift(w)`       | intercept graph `visit`                             |
| `Shared::memoize_by_lift(key)`     | memoise subtree results by key                      |
| `Shared::map_n_bi_lift(co, contra)`| change N (bijection; N is invariant across slots)   |
| `Shared::n_lift(ln, bt, fc)`       | change N with per-slot coordination                 |
| `Shared::explainer_lift()`         | wrap fold with per-node trace recording             |
| `Shared::explainer_describe_lift(fmt, emit)` | streaming trace; `MapR = R`                |
| `Shared::phases_lift(mi, ma, mf)`  | rewrite all three Fold phases (primitive)           |
| `Shared::treeish_lift(mt)`         | rewrite the graph (primitive)                       |

`Local` mirrors the set (except `explainer_describe_lift`), with
Rc storage and no `Send + Sync` bounds.

The last two (`phases_lift`, `treeish_lift`) are the *primitives*:
the per-axis sugars all delegate to one of them. `n_lift` is the
primitive for coordinated N-change; `map_n_bi_lift` is the
bijective special case.

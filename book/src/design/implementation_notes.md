# Implementation notes

Technical specifics of how hylic stores closures, traverses
graphs, and erases types across the lift family.

## Closure storage

The three functions in a `Fold<N, H, R>` (init, accumulate,
finalize) are stored as type-erased closures behind `Arc`:

```rust
{{#include ../../../../hylic/src/domain/shared/fold.rs:fold_struct}}
```

Type erasure (`dyn Fn`) means every `Fold` produced by
`map`/`zipmap` shares the concrete type `Fold<N, H, R>`, so
combinators compose without per-lift type explosion.

`Arc` is required because `Fold` is `Clone` (the lift layer
clones it once per phase closure). `Box<dyn Fn>` is not `Clone`;
`Arc<dyn Fn>` increments a refcount.

The Local domain uses `Rc<dyn Fn>` (lighter refcount,
single-threaded). The Owned domain uses `Box<dyn Fn>` (no
refcount, no `Clone`, single-shot).

`Fold`, `Edgy`, `Graph`, `SeedPipeline`, and related types
implement `Clone` by hand. A derived `Clone` would constrain
type parameters to `Clone`, which the contained `Arc`/`Edgy`/
`Fold` already cover without that bound.

## Graph traversal

`Edgy<N, E>` (and `Treeish<N> = Edgy<N, N>`) stores a
callback-based visit closure:

```rust
{{#include ../../../../hylic/src/graph/edgy.rs:edgy_struct}}
```

The signature is `Fn(&N, &mut dyn FnMut(&E))`. Children are
visited by reference; no allocation per traversal. When a `Vec`
is needed (parallel iteration, for instance), `apply()`
collects via the callback. `Edgy::at(node)` returns a `Visit` —
a zero-allocation push-based iterator with `map`, `filter`,
`fold`, `collect_vec`.

## The `Lift` trait

`Lift<N, N2>` has two GATs: `MapH<H, R>` and `MapR<H, R>`.
`H` and `R` are method-level parameters on `lift_fold<H, R>`,
not trait-level parameters, so they're inferred from the fold
at each call site. The trait is a bifunctor on the `(H, R)`
pair.

Concrete lifts implement `Lift` directly as structs.
`Explainer` is a unit struct; `SeedLift` carries a grow
function and is used internally by `SeedPipeline`. Automatic
composition is provided by a blanket `ComposedLift` impl — no
per-lift boilerplate.

## `ConstructFold`: domain-generic fold construction

`ConstructFold<N>` constructs a `D::Fold<H, R>` from three
closures, generic over `D`. Each domain implements it with its
own storage strategy: Shared wraps in `Arc`, Local in `Rc`.

Shared's fold constructor requires closures to be `Send + Sync`,
but the trait signature is uniform across domains. `make_fold`
is therefore `unsafe fn` with a documented contract — for the
Shared impl, callers must pass closures that are actually
`Send + Sync`. The Shared impl uses `AssertSend<T>` (an
`unsafe`-marked Send+Sync wrapper) with method-call capture
(`.get()`) to satisfy the compiler.

The method-call pattern matters under Rust 2021 precise
captures: `(wrapper.0)(n)` captures the inner field (and
bypasses the Send assertion); `wrapper.get()(n)` captures the
whole wrapper.

Reserved for downstream lift implementations that need
domain-generic fold construction without going through the
typestate pipeline.

## Module visibility

`graph/` is `pub` — it holds the domain-independent graph
types (`Edgy`, `Treeish`, `Graph`) that every other module
imports. `fold/` is `pub(crate)` and contains
domain-independent combinator functions used by the per-domain
`Fold` implementations.

Each domain owns its `Fold` type in
`domain/{shared,local,owned}/fold.rs`. `exec/` and `ops/` are
`pub` — `exec` for executors (`Executor`, `Exec`, `fused`,
`funnel`); `ops` for the operations traits (`FoldOps`,
`TreeOps`) and the lift atoms (`Lift`, `ShapeLift`,
`SeedLift`, …).

## The `prelude` module

Types in `prelude/` are built on the core but optional to use:

- **`VecFold` / `VecHeap`** — convenience fold that collects
  all children before finalizing.
- **`Explainer`** — computation tracing as a `Lift`.
- **`TreeFormatCfg`** — tree-to-string formatting.
- **`Traced`** — path tracking for tree nodes.
- **`memoize_treeish`** — graph-level caching for DAGs.
- **`seeds_for_fallible`** — fallible seed pattern for
  `Either<Error, Valid>` graphs.

## Sibling crates

The following subsystems live in sibling crates and are
documented in their own source:

- **hylic-benchmark** — Rayon executor, Sequential executor,
  benchmark scenarios and runners.
- **hylic-pipeline** — typestate builder over `hylic`'s lift
  primitives. See [Pipelines](../pipeline/overview.md).

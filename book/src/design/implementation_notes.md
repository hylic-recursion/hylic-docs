# Implementation notes

Technical choices in hylic's implementation and the reasoning
behind them.

## Closure storage: `Arc<dyn Fn(...) + Send + Sync>`

The three functions in a `Fold<N, H, R>` (init, accumulate, finalize)
are stored as type-erased closures behind `Arc`:

```rust
pub(crate) impl_init: Arc<dyn Fn(&N) -> H + Send + Sync>,
```

**Why type erasure (`dyn Fn`):** Without it, every `Fold` produced by
`map`/`zipmap` would have a different concrete type. Type erasure lets
the same `Fold<N, H, R>` type hold any combination of closures,
enabling composability.

**Why `Arc`, not `Box`:** `Fold` needs `Clone` for transformation
methods and the Lift layer. `Box<dyn Fn>` is not `Clone`;
`Arc<dyn Fn>` is (atomic reference count increment).

**Local and Owned alternatives:** `local::Fold` uses `Rc<dyn Fn>`
(lighter refcount). `owned::Fold` uses `Box<dyn Fn>` (zero refcount,
but not Clone). The domain system abstracts over this choice.

**Manual `Clone` impls:** `Fold`, `Edgy`, `Graph`, `SeedGraph`, and
`GraphWithFold` all implement `Clone` manually instead of deriving.
The derived `Clone` would require type parameters to be `Clone`, but
the structs only store `Arc`/`Edgy`/`Fold` which are always cloneable.

## Graph traversal: callback-based `Edgy`

`Edgy<N, E>` (and `Treeish<N>`) stores:

```rust
impl_visit: Arc<dyn Fn(&NodeT, &mut dyn FnMut(&EdgeT)) + Send + Sync>
```

**Why callbacks, not `Vec` return:** The original design used
`Fn(&N) -> Vec<E>` — every traversal allocated a `Vec`. The callback
signature `Fn(&N, &mut dyn FnMut(&E))` visits children by reference.
This avoids allocation and cloning.

**`apply()` as escape hatch:** When a `Vec` is actually needed (e.g.,
for parallel iteration), `apply()` collects via the callback.

**`Visit<T, F>` combinator:** `Edgy::at(node)` returns a `Visit` —
a zero-allocation push-based iterator with `map`, `filter`, `fold`,
`collect_vec`.

## Lift: domain-generic, Box storage

`Lift<N, H, R, N2, H2, R2>` stores its four transform closures in
`Box<dyn Fn>`. Lift is not Clone, not Send, not Sync. Its closures
fire once per lifted execution (construction-time transforms, not
per-node operations). Box is the appropriate storage for this.

`LiftOps` is the operations trait parallel to `FoldOps`. The `Lift`
struct implements it.

## `ConstructFold`: domain-generic fold construction

`ConstructFold<N>` constructs a `D::Fold<H, R>` from three closures
generically. Each domain implements it with its own storage strategy:
Shared wraps in Arc, Local wraps in Rc.

The challenge: Shared's fold constructor requires closures to be
Send+Sync, but the trait signature can't vary per domain. Solution:
`make_fold` is `unsafe fn` with a documented contract — for Shared,
closures must actually be Send+Sync. The Shared impl uses
`AssertSend<T>` (unsafe Send+Sync wrapper) with method-call capture
(`.get()`) to satisfy the compiler.

The method-call pattern matters: Rust 2021 precise captures make
`(wrapper.0)(n)` capture the inner field (not the wrapper), bypassing
the Send assertion. `wrapper.get()(n)` forces capture of the whole
wrapper.

Used by ParLazy and ParEager in the `hylic-parallel-lifts` crate.
Not called within hylic core. This is a cross-crate API surface for
one downstream consumer.

## Module visibility

`graph/` is public — it contains the domain-independent graph types
(Edgy, Treeish, Graph, SeedGraph, GraphWithFold) that all code
imports directly. `fold/` is `pub(crate)` — it contains only the
domain-independent combinator functions used internally by the three
domain Fold implementations.

Each domain owns its Fold type in `domain/{shared,local,owned}/fold.rs`.
`cata/` and `ops/` are public — `cata` for executors and lifts,
`ops` for the FoldOps/TreeOps traits.

## The `prelude` module

Types in `prelude/` are built on core but not required to use hylic:

- **VecFold / VecHeap**: Convenience fold that collects all children
  before finalizing.
- **Explainer**: Computation tracing as a Lift.
- **TreeFormatCfg**: Tree-to-string formatting.
- **Traced**: Path tracking for tree nodes.
- **memoize_treeish**: Graph-level caching for DAGs.
- **seeds_for_fallible**: Fallible seed pattern for SeedGraph.

## Sibling crate internals

The following subsystems moved to sibling crates during the crate
split. Their design is documented in those crates' own source:

- **hylic-parallel-lifts**: WorkPool (scoped fork-join pool),
  SyncRef (domain-generic Send+Sync wrapper), ParLazy (two-pass
  parallel evaluation), ParEager (pipelined continuation-passing),
  FoldPtr (lifetime-erased fold operations), Completion/Collector
  (continuation chain)
- **hylic-benchmark**: Rayon executor, Sequential executor,
  HyloSheque (CPS zipper baseline), benchmark scenarios and runners

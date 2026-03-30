# Implementation notes

This section documents the technical choices in hylic's implementation
and the reasoning behind them. It assumes familiarity with Rust's
ownership model, trait objects, and smart pointers.

## Closure storage: `Arc<dyn Fn(...) + Send + Sync>`

The three functions in a `Fold<N, H, R>` (init, accumulate, finalize)
are stored as type-erased closures behind `Arc`:

```rust
pub(crate) impl_init: Arc<dyn Fn(&N) -> H + Send + Sync>,
```

**Why type erasure (`dyn Fn`):** Without it, every `Fold` produced by
`map`/`zipmap` would have a different concrete type (the closure types
change with each transformation). Type erasure lets the same
`Fold<N, H, R>` type hold any combination of closures. This is what
enables composability — you can chain `map().zipmap().map_init()` and
store the result.

**Why `Arc`, not `Box`:** `Fold` needs `Clone` for the transformation
methods (`map`, `zipmap`) and for the graph adapter layer
(`FoldAdapter`, `SeedFoldAdapter`) which clones the fold when building
composed pipelines. `Box<dyn Fn>` is not `Clone`; `Arc<dyn Fn>` is
(via atomic reference count increment). The cost is negligible — a
few atomic increments during pipeline construction, none during
execution.

**Why not `Rc`:** `Arc` enables parallel execution. `Rc` is not
`Send`/`Sync`, so a `Fold` using `Rc` could not be shared across
rayon's thread pool. Since `Arc`'s overhead vs `Rc` is negligible for
this use case (closures are shared, not contended), `Arc` is used
uniformly.

**Why `+ Send + Sync`:** Required by `Arc<T>` — `Arc` only implements
`Send` when `T: Send + Sync`. This bound propagates to all closure
parameters in constructors (`fold()`, `simple_fold()`, `vec_fold()`).
In practice, closures that capture only owned values (`String`, `i32`,
`Arc<...>`) are automatically `Send + Sync`. The bound is cosmetically
noisy but has zero impact on callers.

**Earlier design: `Arc<Box<dyn Fn>>`:** The original implementation
wrapped closures in `Arc<Box<dyn Fn>>` — two layers of indirection.
The `Box` was eliminated: `Arc<dyn Fn>` stores the trait object
directly, saving one pointer hop per call. `Arc::from(Box::new(f) as
Box<dyn Fn(...)>)` handles the unsizing coercion at construction time.

## Graph traversal: callback-based `Edgy`

`Edgy<N, E>` (and `Treeish<N>`) stores:

```rust
impl_visit: Arc<dyn Fn(&NodeT, &mut dyn FnMut(&EdgeT)) + Send + Sync>
```

**Why callbacks, not `Vec` return:** The original signature was
`Fn(&N) -> Vec<E>` — every traversal allocated a `Vec`, cloned
children into it, returned it, then discarded it. For a tree with
`n` nodes, this is `n` allocations per fold execution.

The callback signature `Fn(&N, &mut dyn FnMut(&E))` visits children
by reference. No allocation, no cloning. The producer calls the
callback for each child; the consumer (e.g., `execute`) folds each
child's result immediately.

**Composability preserved:** `map`, `contramap`, `treemap` all compose
callbacks. Each transformation wraps the callback in a new one —
stack-allocated closures, zero heap allocation. For example, `map`
produces new values on the stack and lends them to the next callback:

```rust
fn map(&self, transform) -> Edgy<NodeT, NewEdgeT> {
    edgy_visit(|n, cb| {
        self.visit(n, &mut |e| {
            let mapped = transform(e);  // on stack
            cb(&mapped);                // borrow from stack
        });
    })
}
```

**`apply()` as escape hatch:** When a `Vec` is actually needed (e.g.,
for `par_iter` in parallel traversal), `apply()` collects via the
callback. Parallel execution is the only hot path that uses `apply()`.

**`Visit<T, F>` combinator:** `Edgy::at(node)` returns a `Visit` —
a zero-allocation push-based iterator with `map`, `filter`, `fold`,
`collect_vec`. This provides the composable iteration API without
intermediate `Vec` allocations.

## Resolution children: `Arc<[Resolution]>`

The `Resolution` type (in mb_resolver, the primary consumer) stores
children as `Arc<[Resolution]>` instead of `Vec<Resolution>`:

```rust
pub struct Resolution {
    pub data: HeapData,
    pub children: Arc<[Resolution]>,
}
```

**Why:** `Resolution::get_treeish()` returns a `Treeish` that visits
children by reference. With `Vec<Resolution>`, cloning the `Resolution`
(needed for `Clone` bound in some contexts) deep-copies the entire
subtree — O(n) per clone, O(n²) total for a tree traversal.
`Arc<[Resolution]>` makes clone O(1) — a pointer bump.

**Trade-off:** Building the `Resolution` during the fold requires a
separate `ResolutionHeap` (using `Vec`) during accumulation, converting
to `Arc<[Resolution]>` in the finalize step. This is a one-time cost
per node, paid during construction, not during traversal.

## Execution: separated from algebra

`Fold` has no `execute` method. Execution is a separate concern:

```rust
Strategy::Sequential.run(&fold, &graph, &root)
```

**Why:** The same fold should be runnable with different strategies
(sequential, parallel traversal, lazy parallel fold) without modifying
the algebra. If `execute` were a method on `Fold`, switching strategies
would mean wrapping or replacing the fold.

The `Strategy` enum dispatches to the appropriate execution function.
Adding a new strategy means adding a variant, not changing `Fold`.

## Parallel execution

Three strategies:

- **Sequential** (`cata::sync::run`): Callback-based recursion.
  Zero allocation beyond what the fold itself produces.
- **ParTraverse** (`cata::par_traverse`): Rayon fan-out at each
  recursion level. Uses `apply()` to collect children into a `Vec`
  for `par_iter`. This is the only path that allocates per node.
- **ParFoldLazy** (`cata::par_fold_lazy`): Builds a tree of `UIO<R>`
  (lazy memoized computations). Graph traversal happens inside the
  UIO closures, triggered by `join_par` — making both graph work
  and fold work parallel. Slightly more overhead than ParTraverse
  (one `Arc<OnceLock>` per node) but produces a reifiable
  computation plan.

Benchmarks show ParTraverse and ParFoldLazy perform similarly for
most workloads (within 10-15%), with ParTraverse having less overhead
for CPU-bound work and ParFoldLazy being equivalent for I/O-bound
graph discovery.

## The `prelude` module

Types in `prelude/` are built on core but not required to use hylic:

- `VecFold` / `VecHeap`: Convenience fold that collects all children
  before finalizing. Not always needed — `simple_fold` handles many
  cases without collecting.
- `Explainer`: Wraps a fold to record computation traces. A debugging
  tool, not a core abstraction.
- `TreeFormatCfg`: Tree-to-string formatting. Domain-specific display.
- `Visit`: Push-based iterator. Used internally by `Edgy::at()`.
- `Traced`: Path tracking for tree nodes.
- `memoize_treeish` / `memoize_treeish_by`: Graph-level caching for
  DAGs. Wraps a `Treeish<N>` with a `HashMap<K, Vec<N>>` so repeated
  visits to the same node return cached children. Returns `Treeish<N>`
  — same node type, fold unchanged. `memoize_treeish_by` takes a
  caller-provided key function; `memoize_treeish` uses the node itself
  as key (requires `Hash + Eq`). Not related to `UIO` — `UIO` is for
  parallel execution deferral, not caching.

These are re-exported through `prelude/mod.rs` — e.g.
`hylic::prelude::memoize_treeish` — but live in `prelude/` to keep
the core modules focused.

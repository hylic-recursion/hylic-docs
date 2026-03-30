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
enabling composability — chain `map().zipmap().map_init()` and store
the result.

**Why `Arc`, not `Box`:** `Fold` needs `Clone` for transformation
methods and the adapter layer. `Box<dyn Fn>` is not `Clone`;
`Arc<dyn Fn>` is (atomic reference count increment). The cost is
negligible.

**Why `+ Send + Sync`:** Required by `Arc<T>` — `Arc` only implements
`Send` when `T: Send + Sync`. This propagates to all closure parameters
in constructors. In practice, closures that capture owned values or
`Arc<...>` are automatically `Send + Sync`. The bounds are checked at
construction time; the executor (`Exec`) does not require them.

**Manual `Clone` impls:** `Fold`, `Edgy`, `Graph`, `SeedGraph`, and
`GraphWithFold` all implement `Clone` manually instead of deriving.
The derived `Clone` would require type parameters to be `Clone`, but
the structs only store `Arc`/`Edgy`/`Fold` which are always cloneable.
Manual impls eliminate spurious bounds.

## Graph traversal: callback-based `Edgy`

`Edgy<N, E>` (and `Treeish<N>`) stores:

```rust
impl_visit: Arc<dyn Fn(&NodeT, &mut dyn FnMut(&EdgeT)) + Send + Sync>
```

**Why callbacks, not `Vec` return:** The original design used
`Fn(&N) -> Vec<E>` — every traversal allocated a `Vec`. The callback
signature `Fn(&N, &mut dyn FnMut(&E))` visits children by reference.
No allocation, no cloning.

**`apply()` as escape hatch:** When a `Vec` is actually needed (e.g.,
for parallel iteration in `Exec::rayon()`), `apply()` collects via
the callback.

**`Visit<T, F>` combinator:** `Edgy::at(node)` returns a `Visit` —
a zero-allocation push-based iterator with `map`, `filter`, `fold`,
`collect_vec`.

## Execution: `Exec<N, R>`

`Exec` is parameterized by a single child-visiting lambda:

```rust
// ChildVisitorFn<N, R>: how children are visited and results delivered
pub type ChildVisitorFn<N, R> = dyn Fn(
    &Treeish<N>,                          // graph
    &N,                                    // current node
    &(dyn Fn(&N) -> R + Send + Sync),      // recursive function
    &mut dyn FnMut(&R),                    // result handler
) + Send + Sync;
```

Three constructors produce different lambdas:

- **`Exec::fused()`**: Callback-based recursion via `graph.visit`.
  Recursion and accumulation interleave. Zero allocation.
- **`Exec::sequential()`**: Collects children via `graph.apply`,
  processes one by one. Vec allocation per node.
- **`Exec::rayon()`**: Collects children, `par_iter` for parallel
  recursion. Send + Sync bounds checked at construction, not on Exec.

The recursive function is stack-allocated inside `run()` — captures
only `Arc`-based values (fold, graph, child visitor), so it's `Send +
Sync` without requiring bounds on N, H, or R.

## Resolution children: `Arc<[Resolution]>`

The `Resolution` type (in mb_resolver) stores children as
`Arc<[Resolution]>` instead of `Vec<Resolution>`:

```rust
pub struct Resolution {
    pub data: HeapData,
    pub children: Arc<[Resolution]>,
}
```

`Vec` clone would deep-copy the subtree — O(n) per clone.
`Arc<[Resolution]>` makes clone O(1). Building uses `Vec` during
accumulation, converting to `Arc<[Resolution]>` in finalize.

## The `prelude` module

Types in `prelude/` are built on core but not required to use hylic:

- `VecFold` / `VecHeap`: Convenience fold that collects all children
  before finalizing.
- `Explainer`: Wraps a fold to record computation traces.
- `TreeFormatCfg`: Tree-to-string formatting.
- `Visit`: Push-based iterator. Used internally by `Edgy::at()`.
- `Traced`: Path tracking for tree nodes.
- `memoize_treeish` / `memoize_treeish_by`: Graph-level caching for
  DAGs. Wraps a `Treeish<N>` with `HashMap<K, Vec<N>>` so repeated
  visits return cached children. Same node type — fold unchanged.
- `seeds_for_fallible`: Lifts `Edgy<Valid, Seed>` to
  `Edgy<Either<Err, Valid>, Seed>` for the fallible seed pattern.
  Uses `contramap_or` — a core graph primitive.

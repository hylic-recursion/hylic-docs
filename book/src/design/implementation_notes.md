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
methods and the Lift layer. `Box<dyn Fn>` is not `Clone`;
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
for parallel iteration in `exec::RAYON`), `apply()` collects via
the callback.

**`Visit<T, F>` combinator:** `Edgy::at(node)` returns a `Visit` —
a zero-allocation push-based iterator with `map`, `filter`, `fold`,
`collect_vec`.

## Execution: domain-parameterized executors

The executor is parameterized by a boxing domain:

```rust
pub trait Executor<N: 'static, R: 'static, D: Domain<N>> {
    fn run<H: 'static>(&self, fold: &D::Fold<H, R>, graph: &D::Treeish, root: &N) -> R;
}
```

Each variant is a zero-sized struct `XxxIn<D>(PhantomData<D>)` with
a blanket (or domain-specific) `Executor` impl. The domain marker
lives on the executor type, not on Fold or Treeish. Const values
provide the flattened API: `exec::FUSED`, `exec::RAYON`, etc.

Four variants in `cata/exec/variant/`:

- **`fused/`**: `FusedIn<D>` — callback-based, all domains. Zero
  allocation, zero Arc clones. No Send/Sync/Arc in this module.
- **`sequential/`**: `SequentialIn<D>` — Vec-collect, all domains.
- **`rayon/`**: `RayonIn<D>` — par_iter, Shared only. Needs Sync.
- **`custom/`**: `Custom<N, R>` — user-defined visitor, Shared only.

Each recursion engine takes `&impl FoldOps + &impl TreeOps` — generic
over the operations traits. When called with a concrete user struct,
the compiler monomorphizes and inlines completely.

**`ExecutorExt`**: provides `run_lifted` and `run_lifted_zipped` via
blanket impl for any `Executor<N, R, Shared>`. Lift integration is
Shared-only (Lifts clone Fold/Treeish).

See [Executor architecture](./executors.md) and
[Domain system](./domains.md) for the full design.

## ParRef: lazy memoized computation

`ParRef<T>` wraps a `FnOnce() -> T` with `OnceLock` — computed at most
once, subsequent calls return the cached value. `FnOnce` (not `Fn`)
is the correct trait: the compute closure is consumed on first
evaluation. Internally stored as `Mutex<Option<Box<dyn FnOnce>>>`.

`ParRef::join_par(parrefs)` evaluates a `Vec<ParRef<T>>` in parallel
via rayon's `par_iter`. This is the mechanism behind `ParLazy` —
each node's result is a `ParRef` that, when evaluated, evaluates its
children in parallel first.

## WorkPool: scoped fork-join

`WorkPool` is a fixed-size thread pool created via `WorkPool::with` —
a scoped lifecycle that guarantees all workers are joined on return
(using `std::thread::scope`). No public constructor exists; the pool
cannot escape the closure.

Internally: `Mutex<Vec<Box<dyn FnOnce>>>` work queue + `Condvar` for
worker sleep/wake + `AtomicBool` shutdown flag. Workers loop: pop from
queue or wait on condvar. The calling thread helps drain the queue
while waiting for its children (cooperative scheduling, deadlock-free
for nested fork-join).

## Resolution children: `Arc<[Resolution]>`

The `Resolution` type (in mb_resolver) stores children as
`Arc<[Resolution]>` instead of `Vec<Resolution>`:

`Vec` clone would deep-copy the subtree — O(n) per clone.
`Arc<[Resolution]>` makes clone O(1). Building uses `Vec` during
accumulation, converting to `Arc<[Resolution]>` in finalize.

## The `prelude` module

Types in `prelude/` are built on core but not required to use hylic:

- **VecFold / VecHeap**: Convenience fold that collects all children
  before finalizing.
- **Explainer**: Wraps a fold to record computation traces. Expressed
  as a [Lift](./lifts.md) — `Explainer::lift()` for transparent
  tracing, `Explainer::explain()` for direct trace access.
- **TreeFormatCfg**: Tree-to-string formatting.
- **Traced**: Path tracking for tree nodes.
- **memoize_treeish / memoize_treeish_by**: Graph-level caching for
  DAGs. Same node type — fold unchanged.
- **seeds_for_fallible**: Lifts `Edgy<Valid, Seed>` to
  `Edgy<Either<Err, Valid>, Seed>` for the fallible seed pattern.
- **parallel/**: `ParLazy`, `ParEager`, `WorkPool` — parallel
  execution strategies as Lifts. See [Parallel execution](../cookbook/parallel_execution.md).

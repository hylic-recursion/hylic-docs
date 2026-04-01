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
for parallel iteration in `Exec::rayon()`), `apply()` collects via
the callback.

**`Visit<T, F>` combinator:** `Edgy::at(node)` returns a `Visit` —
a zero-allocation push-based iterator with `map`, `filter`, `fold`,
`collect_vec`.

## Execution: `Executor` trait and variant modules

The executor is a trait with one required method:

```rust
pub trait Executor<N: 'static, R: 'static> {
    fn run<H: 'static>(&self, fold: &Fold<N, H, R>, graph: &Treeish<N>, root: &N) -> R;
    // provided: run_lifted, run_lifted_zipped
}
```

Each variant lives in its own module under `cata/exec/variant/`:

- **`variant/fused/`**: `Fused` — callback-based recursion via
  `graph.visit`. Zero allocation, zero Arc clones. Fold and graph are
  passed by `&` reference through the entire recursion. **No Send,
  Sync, or Arc appears in this module's code** — all thread-boundary
  concerns are absent.

- **`variant/custom/`**: `Custom<N, R>` — wraps a `ChildVisitorFn`
  that controls how children are visited. Send + Sync bounds are
  contained here. Two built-in constructors:
  - `Custom::sequential()` — collect children to Vec, iterate (N: Clone)
  - `Custom::rayon()` — collect children, par_iter (N: Clone+Send+Sync)

The `Exec<N, R>` enum wraps these variants for everyday runtime
dispatch. Its API is identical to pre-trait usage (`Exec::fused()`,
`Exec::rayon()`, `exec.run(...)`). Inherent methods on the enum
shadow the trait methods, so no `use Executor` import is needed.

For zero-overhead static dispatch, use variant types directly:
`Fused.run(...)` or `Custom::rayon().run(...)`.

**`run_lifted`**: Provided by the trait — implementors get it free.
Accepts a [Lift](./lifts.md), transforms fold + treeish, runs the
lifted computation, unwraps. This is how parallel strategies and
the Explainer integrate.

**Adding a new executor:** Create a directory `variant/<name>/`,
define a struct, implement `Executor<N, R>`. Only `run()` is
required; `run_lifted` and `run_lifted_zipped` are provided
automatically. Add the variant to the `Exec` enum and its inherent
method dispatchers.

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

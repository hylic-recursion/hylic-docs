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
No allocation, no cloning.

**`apply()` as escape hatch:** When a `Vec` is actually needed (e.g.,
for parallel iteration), `apply()` collects via the callback.

**`Visit<T, F>` combinator:** `Edgy::at(node)` returns a `Visit` —
a zero-allocation push-based iterator with `map`, `filter`, `fold`,
`collect_vec`.

## Inherent methods on executors

Every executor const (`dom::FUSED`, `dom::RAYON`, etc.) has inherent
`run`, `run_lifted`, `run_lifted_zipped` — no trait import needed.
The trick: D is constrained by the self type (fixed by the const's
type), while N, H, R are inferred from arguments:

```rust
impl<D> FusedIn<D> {
    pub fn run<N, H, R>(&self, fold: &<D as Domain<N>>::Fold<H, R>, ...)
    where D: Domain<N> { ... }
}
```

The `Executor` trait exists for generic code (pipeline.rs, user code
that accepts any executor). The inherent methods delegate to the trait
impl internally.

## SyncRef: domain-generic parallelism

`SyncRef<'a, T>` wraps a reference and declares it `Send + Sync`:

```rust
pub struct SyncRef<'a, T: ?Sized>(pub &'a T);
unsafe impl<T: ?Sized> Sync for SyncRef<'_, T> {}
unsafe impl<T: ?Sized> Send for SyncRef<'_, T> {}
```

**The problem:** `Rc<dyn Fn>` is `!Sync`, so `&Rc<dyn Fn>` is `!Send`.
A Local-domain fold can't be passed to pool worker threads.

**The solution:** `WorkPool` uses `std::thread::scope` — all workers
join before the scope exits. Within the scope, borrowed data outlives
all workers. Workers only deref + call through the reference — no Rc
cloning, no mutation of refcounts. SyncRef makes this safe by declaring
the borrow Send+Sync.

**Where it's used:**
- `PoolIn<D>` wraps `&fold` and `&graph` in SyncRef before passing to
  the recursion engine
- `fork_join_map` wraps the items slice in SyncRef internally, removing
  the `T: Sync` bound from the public API
- ParEager's Collector receives fold operations via FoldPtr
  (lifetime-erased raw pointer), enabling domain-generic operation

## WorkPool: scoped fork-join

`WorkPool` is a fixed-size thread pool created via `WorkPool::with` —
a scoped lifecycle that guarantees all workers are joined on return
(using `std::thread::scope`). No public constructor; the pool
cannot escape the closure.

Internally: crossbeam-deque's lock-free `Injector` for task
distribution + `Condvar` for worker sleep/wake + `AtomicBool`
shutdown flag. A `ShutdownGuard` (Drop impl) ensures workers wake
even if the user closure panics.

### pool.join(f1, f2) -> (A, B)

Scoped fork-join: submit f2 to the pool, run f1 on the caller's
thread, spin-help until f2 completes. Both closures may borrow from
the caller's stack — lifetime erasure via `std::mem::transmute`
converts `Box<dyn FnOnce() + Send + '_>` to `'static`. Safe because
join() blocks until f2 completes (the stack frame outlives both
closures).

**JoinSlot pattern:** Raw pointers to the stack are bundled in a
`JoinSlot` struct that implements `Send`. A method call
(`slot.complete(r)`) ensures Rust 2021 precise captures grab the
whole struct (which is Send) rather than individual raw pointer fields
(which aren't).

**Panic safety:** f2 is wrapped in `catch_unwind`. If it panics, the
payload is stored and re-raised via `resume_unwind` on the caller's
thread.

### fork_join_map(pool, items, f, depth, max_depth) -> Vec<R>

Binary-split recursive fork-join over a slice. Recursively halves the
work using join() at each level. Sequential below max_depth or when
one item remains. Preserves order.

No `T: Sync` bound — the slice is wrapped in SyncRef internally.

## ParLazy: two-pass parallel evaluation

ParLazy builds a data tree of `LazyNode<H, R>` (Phase 1), then
evaluates bottom-up via `fork_join_map` (Phase 2). Each LazyNode
stores its heap value, child handles (`Arc<LazyNode>`), and an
`OnceLock<R>` for memoized results.

Phase 2 borrows the fold through `SyncRef` — no fold closures are
captured in the tree nodes. This data-tree decoupling is what makes
ParLazy domain-generic (Shared and Local).

## Continuation-passing in ParEager

ParEager's pipelined execution uses Completion + Collector + FoldPtr:

- **Completion<R>**: one-shot result slot with a type-erased parent
  callback (`Box<dyn FnOnce(R) + Send>`). When set, calls the callback.
- **Collector<H, R>**: atomic countdown over children. The last child
  to arrive runs the parent's acc+fin inline — no task submission, no
  blocking.
- **FoldPtr**: lifetime-erased raw pointer to fold operations
  (accumulate + finalize). Collector receives fold operations through
  FoldPtr rather than capturing domain-specific closures (Arc for
  Shared, Rc for Local). This is what makes ParEager domain-generic.

Type erasure via the callback closure: H is captured inside the
closure, invisible to Completion (which only knows R). FoldPtr
carries the fold's accumulate/finalize as a raw pointer valid for
the duration of Phase 2, allocated once per non-leaf child.

No task ever blocks. The chain propagates upward: leaf completes,
notifies parent, parent completes, notifies grandparent, up to root.

### Collector's manual Send+Sync

`Collector<N, H, R>` holds `FoldPtr<N, H, R>` + `Mutex<H>` +
`Mutex<Vec<Option<R>>>` + `Completion<R>` + `AtomicUsize`. The
auto-derived Send would require `N: Send` (from FoldPtr's phantom
type in the trait-object pointer). But N is never accessed as a
value — it's only in the fat pointer's vtable type. So we write:

```rust
unsafe impl<N, H: Send, R: Send> Send for Collector<N, H, R> {}
unsafe impl<N, H: Send, R: Send> Sync for Collector<N, H, R> {}
```

H: Send and R: Send are required (actual data crossing threads).
N: Send is not (phantom in the vtable pointer).

### Height-based cutoff in EagerSpec

`EagerSpec.min_height_to_fork` controls when to create a Collector
vs accumulate inline. Height = distance from leaves, computed
naturally: each `EagerResult` carries its subtree height, parent
computes `max(child_heights) + 1`.

Height is better than depth for parallelism cutoff. A height-2
cutoff means "sequential for subtrees with ≤ b² nodes" regardless
of position in the tree. Depth-based cutoff requires knowing total
tree size to set a useful threshold — a fixed depth that works for
200-node trees is wrong for 5000-node trees.

Note: this is different from `PoolSpec.fork_depth_limit` which IS
depth-based. PoolSpec tracks depth in the recursion engine (passed
as a parameter), not in the fold. The two cutoffs are independent
and complementary when combining `PoolIn + ParEager`.

## Lift: domain-generic, Box storage

`Lift<D, N, H, R, N2, H2, R2>` stores its four transform closures
in `Box<dyn Fn>` — not Arc. Lift is not Clone, not Send, not Sync.
Its closures fire once per `run_lifted` call (construction-time
transforms, not per-node operations). Box is the correct storage.

`LiftOps` is the operations trait parallel to `FoldOps`. The `Lift`
struct implements it. `map_lifted_fold` and `map_lifted_treeish` take
self by value (consume the Lift to produce a new one).

## ConstructFold: domain-generic fold construction

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
wrapper. Same principle as the JoinSlot pattern in pool.join().

## Fold stash: passing fold across Lift phases

ParLazy and ParEager need the original fold in both `lift_fold`
(Phase 1 setup) and `unwrap` (Phase 2 evaluation). The Lift's
closure signatures don't connect these. Solution: a shared cell
`Rc<RefCell<Option<D::Fold<H, R>>>>` — `lift_fold` writes, `unwrap`
reads. Both run on the same thread (sequentially, during run_lifted).
Box<dyn Fn> in the Lift doesn't require Send, so Rc is fine.

## Data tree decoupling in parallel lifts

ParLazy and ParEager build data trees (Node<H, R>) during Phase 1
rather than closure trees. Nodes store only the
heap value and child handles — no fold closures captured. Phase 2
receives the fold externally via SyncRef (for lazy evaluation) or
FoldPtr (for eager pipelining).

This decoupling (data nodes ↔ fold operations) is what makes
domain-generic parallel lifts possible. Without it, each node would
capture domain-specific closures (Arc for Shared, Rc for Local),
and the Rc closures couldn't cross thread boundaries.

## pub(crate) on implementation modules

`fold/`, `graph/`, `pipeline.rs`, `parref.rs` are `pub(crate)` —
internal to the hylic crate. Everything is re-exported through domain
modules (`domain::shared`, `domain::local`, `domain::owned`). One way
in for users.

`cata/` and `ops/` remain public — `cata` for Lift, `ops` for the
FoldOps/TreeOps/LiftOps traits needed by generic code.

## Resolution children: `Arc<[Resolution]>`

The `Resolution` type (in mb_resolver) stores children as
`Arc<[Resolution]>` instead of `Vec<Resolution>`. `Vec` clone
deep-copies the subtree — O(n). `Arc<[Resolution]>` clone is O(1).

## The `prelude` module

Types in `prelude/` are built on core but not required to use hylic:

- **VecFold / VecHeap**: Convenience fold that collects all children
  before finalizing.
- **Explainer**: Computation tracing as a Lift.
- **TreeFormatCfg**: Tree-to-string formatting.
- **Traced**: Path tracking for tree nodes.
- **memoize_treeish**: Graph-level caching for DAGs.
- **seeds_for_fallible**: Fallible seed pattern for SeedGraph.
- **parallel/**: `ParLazy`, `ParEager`, `WorkPool` — parallel
  execution strategies as Lifts.

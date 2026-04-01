# Executor architecture

The executor controls **how** the tree recursion runs. The fold says
what to compute; the treeish says where the children are; the executor
decides the traversal order and parallelism strategy.

## The `Executor` trait

```rust
pub trait Executor<N: 'static, R: 'static> {
    fn run<H: 'static>(
        &self, fold: &Fold<N, H, R>, graph: &Treeish<N>, root: &N
    ) -> R;

    // Provided:
    fn run_lifted(...)  -> R0 { /* uses self.run() */ }
    fn run_lifted_zipped(...) -> (R0, R) { /* uses self.run() */ }
}
```

One required method. Two provided. Any type that implements `run()`
automatically gets Lift integration — all parallel strategies and
the Explainer work through `run_lifted`, which calls `run()` on the
lifted fold.

The trait has minimal bounds: `N: 'static, R: 'static`. No `Send`,
no `Sync`, no `Clone`. Each executor impl adds only the bounds it
actually needs.

## Built-in executors

Each lives in its own module under `cata/exec/variant/`. Each is a
zero-sized struct with its own recursion engine.

### Fused

```rust
Fused.run(&fold, &graph, &root)
```

Callback-based traversal via `graph.visit()`. Recursion and
accumulation interleave inside the callback — no collection step,
no allocation, no Arc clones. The fold and graph are passed by
`&` reference through the entire recursion.

**Bounds:** `N: 'static, R: 'static` — nothing more. The words
`Send`, `Sync`, and `Arc` do not appear in this module's code.

**When to use:** Sequential computation where you want minimal
overhead. The fastest single-threaded path.

### Sequential

```rust
Sequential.run(&fold, &graph, &root)
```

Collects children to a `Vec` via `graph.apply()`, then iterates
sequentially. Same result as Fused but uses the unfused traversal
pattern — exists as a reference implementation for the `apply()` path.

**Bounds:** `N: Clone + 'static` (for `apply()`'s `collect_vec()`).

**When to use:** Testing the Vec-collection code path. Prefer Fused
for production sequential work.

### Rayon

```rust
Rayon.run(&fold, &graph, &root)
```

Collects children to a `Vec`, then `par_iter()` for parallel
recursion via rayon's work-stealing thread pool. Like Fused, it
passes fold and graph by `&` reference — rayon's scoped parallelism
guarantees the references are valid across threads. Zero Arc clones
in recursion.

**Bounds:** `N: Clone + Send + Sync, R: Send + Sync` — contained
in this module's impl. The `Executor` trait itself doesn't mention them.

**When to use:** Parallel computation on CPU-bound workloads. The
simplest parallel executor, backed by rayon's mature scheduler.

### Custom

```rust
let visitor = Arc::new(|graph, node, recurse, handle| { ... });
Custom::new(visitor).run(&fold, &graph, &root)
```

User-defined child visitor. The `ChildVisitorFn` closure controls
how children are traversed and how results are delivered. This is the
escape hatch for parallelism strategies that don't fit the built-in
executors.

**Cost:** 5 Arc clones per node (fold ×3, graph, visitor) due to the
type-erased closure capture. The built-in executors avoid this by using
dedicated recursion engines.

**When to use:** Custom parallelism strategies, testing, research.

## The `Exec` enum

For runtime dispatch — choosing an executor at runtime:

```rust
let exec = Exec::fused();    // or Exec::rayon(), Exec::sequential()
exec.run(&fold, &graph, &root);
```

`Exec<N, R>` wraps all built-in variants. Its `run()` method does a
`match` to dispatch to the concrete variant's `run()`. The API is
unchanged from pre-trait hylic — `Exec::fused()` and `Exec::rayon()`
work identically.

**Bounds on Exec::run():** `N: Clone + Send + Sync, R: Send + Sync`
— the union of all variants' requirements. Code that doesn't need
these bounds should use variant types directly: `Fused.run(...)` or
`Rayon.run(...)`.

## Choosing static vs runtime dispatch

| Approach | Syntax | Overhead | Bounds |
|----------|--------|----------|--------|
| Direct variant | `Fused.run(...)` | zero (inlined) | variant-specific |
| Exec enum | `Exec::fused().run(...)` | one match branch | union of all |
| Generic | `fn go(e: &impl Executor<N,R>)` | zero (monomorphized) | trait-level only |

Most code should use direct variants (`Fused`, `Rayon`). The `Exec`
enum is for cases where the executor is determined at runtime
(configuration, user input). Generic functions accept any executor
via `&impl Executor<N, R>`.

## Adding a new executor

1. Create `cata/exec/variant/<name>/mod.rs`
2. Define a struct (typically zero-sized)
3. Implement `Executor<N, R>` — only `run()` is required
4. Add the variant to the `Exec` enum and its dispatchers
5. Add a convenience constructor on `Exec` if appropriate

The provided `run_lifted` and `run_lifted_zipped` methods work
automatically — the new executor integrates with all Lifts (ParLazy,
ParEager, Explainer) without additional code.

## Design principles

**Bounds are contained.** The `Executor` trait requires only `'static`.
Each variant's module adds only the bounds it needs. `Send + Sync`
appears exclusively in modules that cross thread boundaries (Rayon,
Custom). The Fused module has zero mentions of `Send`, `Sync`, or `Arc`
in its code.

**Recursion engines are dedicated.** Each built-in executor has its own
recursive function rather than routing through a shared abstraction.
This eliminates the Arc clone overhead that a shared
`ChildVisitorFn`-based approach would impose. The three built-in
variants (`Fused`, `Sequential`, `Rayon`) are all zero-sized with
zero per-node allocation or reference counting.

**The trait enables extension.** Parallelism libraries beyond rayon
(custom thread pools, async runtimes, GPU dispatch) integrate by
implementing `Executor`. The trait is the stable contract; the
variant modules are the implementations. New strategies don't require
changes to the trait, to Fold, to Treeish, or to any existing executor.

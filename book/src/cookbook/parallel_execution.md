# Parallel execution

hylic provides two built-in executors. `dom::FUSED` runs the fold
sequentially through callback-based recursion. The Funnel executor
parallelizes the same fold across a scoped thread pool. Both are
invoked through the same `.run()` method.

## Sequential: `dom::FUSED`

Callback-based recursion on a single thread, with no overhead beyond
the fold closures themselves:

```rust
use hylic::domain::shared as dom;
dom::FUSED.run(&fold, &graph, &root);
```

## Parallel: Funnel

The Funnel executor preserves the fused property — children are
discovered through `graph.visit` and processed concurrently. No
intermediate tree is built.

### One-shot

```rust
use hylic::cata::exec::funnel;
use hylic::domain::shared as dom;

dom::exec(funnel::Spec::default(8)).run(&fold, &graph, &root);
```

`Spec::default(n)` uses the Robust policy preset. `.run()` creates
a scoped thread pool internally, runs the fold, and joins before
returning.

### Session scope

For repeated folds, amortize pool creation:

```rust
dom::exec(funnel::Spec::default(8)).session(|s| {
    s.run(&fold1, &graph1, &root1);
    s.run(&fold2, &graph2, &root2);
});
```

The pool lives for the closure. Each `.run()` inside is cheap.

### Explicit attach

Provide the pool yourself:

```rust
funnel::Pool::with(8, |pool| {
    let pw = dom::exec(funnel::Spec::default(8)).attach(pool);
    let sh = dom::exec(funnel::Spec::for_wide_light(8)).attach(pool);
    pw.run(&fold, &graph, &root);
    sh.run(&fold, &graph, &root);
});
```

Different policies can share a pool — each `.attach()` consumes a
(Copy) Spec and binds it to the pool, producing a session-level
executor.

### Policy variants

| Preset | Best for |
|---|---|
| `Spec::default(n)` | General purpose |
| `Spec::for_wide_light(n)` | Wide trees (bf > 10) |
| `Spec::for_deep_narrow(n)` | Deep chains (bf = 2) |
| `Spec::for_low_overhead(n)` | Overhead-sensitive |
| `Spec::for_high_throughput(n)` | Heavy balanced |

See [Funnel policies](../funnel/policies.md) for the full decision
guide and [The Exec pattern](../executor-design/exec_pattern.md) for
the type-level design behind `.run()`, `.session()`, and `.attach()`.

## External parallel options

Two additional strategies live in sibling crates:

- **Rayon** (hylic-benchmark): `par_iter`-based fork-join
- **Parallel lifts** (hylic-parallel-lifts): `ParLazy` and `ParEager`

## Working example

This example uses a flat adjacency list — nodes are integer indices,
children are looked up by index. The same fold runs sequentially
(Fused) and in parallel (Funnel) with identical results.

```rust
{{#include ../../../src/cookbook/parallel_execution.rs}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__parallel_execution__tests__parallel.snap:5:}}
```

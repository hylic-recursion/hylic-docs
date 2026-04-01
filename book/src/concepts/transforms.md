# Transformations and Lifts

Because Fold and Treeish are data — closures behind Arc — you
transform them by producing new values with modified closures.
No subclassing, no wrapping, no runtime dispatch.

## Fold transformations

A `Fold<N, H, R>` has three closures. Each can be wrapped
independently:

```dot process
digraph {
    rankdir=LR;
    node [shape=box, style="rounded,filled", fillcolor="#f5f5f5", fontname="monospace", fontsize=11];
    edge [fontname="sans-serif", fontsize=10];

    F1 [label="Fold<N, H, R>"];
    F2 [label="+ logging\nFold<N, H, R>"];
    F3 [label="+ extra result\nFold<N, H, (R, X)>"];
    F4 [label="different node\nFold<N2, H, R>"];
    F5 [label="two folds at once\nFold<N, (H1,H2), (R1,R2)>"];

    F1 -> F2 [label="map_init"];
    F1 -> F3 [label="zipmap"];
    F1 -> F4 [label="contramap"];
    F1 -> F5 [label="product"];
}
```

| Combinator | What it does |
|---|---|
| `map_init(f)` | Wrap the init phase: `f(orig_init)` → new init |
| `map_accumulate(f)` | Wrap the accumulate phase |
| `map_finalize(f)` | Wrap the finalize phase |
| `map(fwd, back)` | Change result type: `R → R2` (with back-mapping for accumulate) |
| `zipmap(f)` | Augment result: `R → (R, Extra)` |
| `contramap(f)` | Change node type: `Fold<N,...> → Fold<N2,...>` |
| `product(other)` | Two folds in one traversal: `(R1, R2)` |

Each returns a new `Fold` — the original is unchanged. Example:

```rust
// Add logging to any fold, without modifying it
let logged = fold.map_init(|orig| Box::new(move |n: &Dir| {
    println!("visiting {}", n.name);
    orig(n)
}));

// Two independent folds in one pass
let both = size_fold.product(&depth_fold());
let (total_size, max_depth) = exec.run(&both, &graph, &root);
```

## Treeish transformations

`Treeish<N>` (which is `Edgy<N, N>`) supports:

| Combinator | What it does |
|---|---|
| `map(f)` | Transform edges: `Edgy<N, E> → Edgy<N, E2>` |
| `contramap(f)` | Change node type: `Edgy<N, E> → Edgy<N2, E>` |
| `contramap_or(f)` | Change node type, with fallback edges |
| `filter(pred)` | Keep only edges matching predicate |
| `treemap(co, contra)` | Both map + contramap (for `Treeish<N> → Treeish<N2>`) |

## When transformations aren't enough: Lift

Fold transformations change how the three phases behave, but
they preserve the types `H` and `R`. What if you need a different
heap type — one that carries trace information, or collects
deferred computations?

A **Lift** transforms both the Treeish and the Fold into a
different type domain, runs the computation there, and maps the
result back:

```rust
{{#include ../../../../hylic/src/cata/lift.rs:lift_struct}}
```

Four functions:
- `lift_treeish`: `Treeish<N> → Treeish<N2>`
- `lift_fold`: `Fold<N,H,R> → Fold<N2,H2,R2>`
- `lift_root`: `&N → N2`
- `unwrap`: `R2 → R`

Execution goes through `Exec::run_lifted`:

```rust
{{#include ../../../../hylic/src/cata/exec/mod.rs:run_lifted}}
```

The pattern: lift types → run in lifted domain → unwrap back to R.
The caller gets the same `R` as if no Lift were applied.

```dot process
digraph {
    rankdir=LR;
    node [shape=box, style="rounded,filled", fillcolor="#f5f5f5", fontname="monospace", fontsize=11];
    edge [fontname="sans-serif", fontsize=10];

    subgraph cluster_orig {
        label="Original";
        style=dashed; color="#999999"; fontname="sans-serif";
        F [label="Fold<N, H, R>"];
        T [label="Treeish<N>"];
    }

    subgraph cluster_lifted {
        label="Lifted domain";
        style=solid; color="#333333"; fontname="sans-serif";
        F2 [label="Fold<N2, H2, R2>"];
        T2 [label="Treeish<N2>"];
    }

    R2 [label="R2", shape=ellipse];
    R  [label="R", shape=ellipse, style=filled, fillcolor="#d4edda"];

    F -> F2 [label="lift_fold"];
    T -> T2 [label="lift_treeish"];
    F2 -> R2 [label="exec.run"];
    R2 -> R  [label="unwrap"];
}
```

## The three built-in Lifts

hylic provides three Lifts, each transforming the fold's type
domain for a different purpose:

### Explainer — computation tracing

`Explainer::lift()` wraps the fold to record every accumulation
step. The heap becomes `ExplainerHeap` (initial state, node,
transitions, working state). The result becomes `ExplainerResult`
(original result + full trace). Unwrap extracts `R`.

```rust
use hylic::cata::exec::{self, Executor, ExecutorExt};

// Transparent: get R, trace discarded
let r = exec::FUSED.run_lifted(&Explainer::lift(), &fold, &graph, &root);

// With callback: inspect trace before extracting R
let r = exec::FUSED.run_lifted(
    &Explainer::lift_with(|trace| println!("{:?}", trace)),
    &fold, &graph, &root,
);

// Zipped: get both R and the full ExplainerResult
let (r, trace) = exec::FUSED.run_lifted_zipped(&Explainer::lift(), &fold, &graph, &root);
```

In recursion-scheme terms, this is a histomorphism — each node
sees its subtree's full computation history.

### ParLazy — lazy parallel evaluation

`ParLazy::lift()` transforms the fold so each node's result is a
`ParRef<R>` — a deferred computation. The executor builds a tree
of `ParRef` values (Phase 1). Unwrap calls `eval()` on the root,
which triggers bottom-up parallel evaluation via rayon (Phase 2).

```rust
let r = exec::FUSED.run_lifted(&ParLazy::lift(), &fold, &graph, &root);
```

### ParEager — fork-join parallelism

`ParEager::lift(pool)` extracts heaps into an `EagerNode` tree
(Phase 1), then executes bottom-up with a hand-rolled fork-join
scheduler backed by a `WorkPool` (Phase 2).

```rust
WorkPool::with(WorkPoolSpec::threads(3), |pool| {
    exec::FUSED.run_lifted(&ParEager::lift(pool), &fold, &graph, &root)
});
```

See [Parallel execution](../cookbook/parallel_execution.md) for
detailed flow diagrams and benchmarks.

## The mathematical picture

A Fold is an F-algebra: a function `F<R> → R` that collapses one
layer of structure. hylic decomposes it into three phases
(init/accumulate/finalize) through the intermediate heap type `H`.

A Lift is a natural transformation between two F-algebras. It maps
the carrier types `(H, R)` to `(H2, R2)` while preserving the
fold structure. The `unwrap` function projects back: `R2 → R`.
The computation produces the same result regardless of which
algebra it runs in — the Lift is transparent.

This is why you can add tracing, parallelism, or any other
enrichment without the caller knowing. The fold's structure is
preserved. Only the domain it runs in changes.

# Lifts: cross-cutting concerns

A Lift transforms both the Fold and Treeish into a different type
domain, runs the computation there, and maps the result back. The
caller gets the same `R` — the Lift is transparent.

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

Lifts are Shared-domain only — they clone Fold and Treeish (which
requires Arc-based types). Use `exec::FUSED.run_lifted(...)` or
any Shared-domain executor.

## Explainer — computation tracing

Records every step of the fold at every node: the initial heap,
each child result accumulated, and the final result. A histomorphism
— each node sees its subtree's full computation history.

```rust
use hylic::cata::exec::{self, Executor, ExecutorExt};
use hylic::prelude::Explainer;

// Transparent: get R, trace discarded
let r = exec::FUSED.run_lifted(&Explainer::lift(), &fold, &graph, &root);

// With callback: inspect trace at each node
let r = exec::FUSED.run_lifted(
    &Explainer::lift_with(|trace| eprintln!("trace: {:?}", trace)),
    &fold, &graph, &root,
);

// Zipped: get both R and the full ExplainerResult
let (r, trace) = exec::FUSED.run_lifted_zipped(
    &Explainer::lift(), &fold, &graph, &root
);
```

The `ExplainerResult` contains the original result plus the full
`ExplainerHeap` — initial heap, node, transitions (each with the
incoming child result and resulting heap state), and working heap.

Use the Explainer for debugging, visualization, or understanding
how a fold processes a specific tree.

## ParLazy — lazy parallel evaluation

Transforms the fold so each node's result is a `ParRef<R>` — a
lazy, memoized computation. Phase 1 builds the ParRef tree (cheap).
Phase 2 evaluates bottom-up via rayon's `par_iter`.

```rust
use hylic::prelude::ParLazy;

let r = exec::FUSED.run_lifted(&ParLazy::lift(), &fold, &graph, &root);
```

```dot process
digraph {
    rankdir=LR;
    node [shape=box, style="rounded,filled", fillcolor="#f5f5f5", fontname="monospace", fontsize=11];
    edge [fontname="sans-serif", fontsize=10];

    subgraph cluster_p1 {
        label="Phase 1: build ParRef tree";
        style=dashed; color="#999999"; fontname="sans-serif";
        build [label="init → (H, [])\nacc → push ParRef\nfin → create ParRef closure"];
    }

    subgraph cluster_p2 {
        label="Phase 2: evaluate";
        style=solid; color="#333333"; fontname="sans-serif";
        eval [label="root.eval()\n→ join_par(children)\n→ accumulate + finalize"];
    }

    build -> eval [label="unwrap calls eval()", style=dashed];
}
```

Best when: init is expensive (parallelized in Phase 2), accumulate
and finalize are cheap. The parallelism comes from rayon's `par_iter`
inside `ParRef::join_par`.

## ParEager — fork-join parallelism

Extracts heaps into an `EagerNode` tree (Phase 1), then executes
bottom-up with fork-join via a `WorkPool` (Phase 2).

```rust
use hylic::prelude::{ParEager, WorkPool, WorkPoolSpec};

WorkPool::with(WorkPoolSpec::threads(3), |pool| {
    exec::FUSED.run_lifted(&ParEager::lift(pool), &fold, &graph, &root)
});

// Convenience form:
ParEager::with(WorkPoolSpec::threads(3), |lift| {
    exec::FUSED.run_lifted(lift, &fold, &graph, &root)
});
```

```dot process
digraph {
    rankdir=LR;
    node [shape=box, style="rounded,filled", fillcolor="#f5f5f5", fontname="monospace", fontsize=11];
    edge [fontname="sans-serif", fontsize=10];

    subgraph cluster_p1 {
        label="Phase 1: extract heaps";
        style=dashed; color="#999999"; fontname="sans-serif";
        build [label="init → EagerNode{heap, []}\nacc → push Arc<EagerNode>\nfin → wrap into EagerHandle"];
    }

    subgraph cluster_p2 {
        label="Phase 2: fork-join";
        style=solid; color="#333333"; fontname="sans-serif";
        fork [label="submit n-1 children to pool\nrun last child in-place\nhelp process queue\naccumulate + finalize"];
    }

    build -> fork [label="unwrap triggers exec_node()", style=dashed];
}
```

Best when: you want explicit control over the thread pool (fixed
thread count, scoped lifecycle). The `WorkPool` uses `std::thread::scope`
— workers are guaranteed joined on return.

## Combining executors with Lifts

Any combination works — the executor controls Phase 1, the Lift
controls Phase 2:

| | exec::FUSED | exec::RAYON |
|---|---|---|
| ParLazy | Sequential build → rayon eval | Parallel build → rayon eval |
| ParEager | Sequential build → pool fork-join | Parallel build → pool fork-join |
| Explainer | Sequential trace | Parallel trace |

`exec::RAYON` + Lift = double parallelism: Phase 1 is parallel (rayon),
Phase 2 is also parallel (rayon or pool). This is useful for large
trees where both phases benefit from parallelism.

## Writing your own Lift

A Lift is four functions:

```rust
use hylic::cata::Lift;

let my_lift = Lift::new(
    |treeish| treeish,                      // lift_treeish: Treeish<N> → Treeish<N2>
    |fold| transform_fold(fold),            // lift_fold: Fold<N,H,R> → Fold<N2,H2,R2>
    |root| root.clone(),                    // lift_root: &N → N2
    |lifted_result| extract(lifted_result), // unwrap: R2 → R
);

let r = exec::FUSED.run_lifted(&my_lift, &fold, &graph, &root);
```

The key constraint: `unwrap(run(lift_fold(fold), lift_treeish(graph), lift_root(root)))`
must produce the same `R` as `run(fold, graph, root)`. The Lift is
transparent — it doesn't change the answer, only how it's computed.

Common patterns:
- **Identity treeish**: `|t| t` — don't change the tree, only the fold
- **Wrapping fold**: the fold's H2 contains the original H plus extra state
- **Deferred result**: R2 is a lazy handle (ParRef, EagerHandle) that
  produces R on unwrap

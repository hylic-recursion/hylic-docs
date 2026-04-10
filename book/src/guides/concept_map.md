# Concept map

How the pieces fit together.

## The three axes

hylic is built on three orthogonal axes. Each can be chosen
independently:

```dot process
digraph {
    rankdir=TB;
    node [shape=box, style="rounded,filled", fontname="sans-serif", fontsize=11];
    edge [fontname="sans-serif", fontsize=10];

    subgraph cluster_ops {
        label="WHAT to compute";
        style=filled; fillcolor="#d4edda22"; color="#28a745";
        fontname="sans-serif"; fontsize=12;

        foldops [label="FoldOps\ninit / accumulate / finalize", fillcolor="#d4edda"];
        treeops [label="TreeOps\nvisit (children)", fillcolor="#d4edda"];
    }

    subgraph cluster_domain {
        label="HOW closures are boxed";
        style=filled; fillcolor="#fff3cd22"; color="#ffc107";
        fontname="sans-serif"; fontsize=12;

        shared [label="Shared\nArc<dyn Fn + Send + Sync>", fillcolor="#fff3cd"];
        local [label="Local\nRc<dyn Fn>", fillcolor="#fff3cd"];
        owned [label="Owned\nBox<dyn Fn>", fillcolor="#fff3cd"];
    }

    subgraph cluster_exec {
        label="HOW to traverse";
        style=filled; fillcolor="#cce5ff22"; color="#004085";
        fontname="sans-serif"; fontsize=12;

        fused [label="Fused\nzero-overhead callback", fillcolor="#cce5ff"];
        funnel_e [label="Funnel\nCPS work-stealing\n(Shared)", fillcolor="#cce5ff"];
    }

    foldops -> shared [style=invis];
    treeops -> local [style=invis];
    shared -> fused [style=invis];
}
```

**Operations** define the computation. **Domain** determines boxing
overhead. **Executor** determines traversal strategy. Any combination
works (subject to domain compatibility).

## Type landscape

<!-- -->

```dot process
digraph {
    rankdir=TB;
    node [shape=box, style="rounded,filled", fillcolor="#f5f5f5", fontname="monospace", fontsize=10];
    edge [fontname="sans-serif", fontsize=9];

    subgraph cluster_traits {
        label="Operations traits (ops::)";
        style=dashed; color="#999999";
        fontname="sans-serif";

        FoldOps [label="FoldOps<N, H, R>\ninit / accumulate / finalize"];
        TreeOps [label="TreeOps<N>\nvisit / apply"];
        LiftOps [label="LiftOps<D, N, H, R, ...>\nlift_fold / lift_treeish / unwrap"];
    }

    subgraph cluster_concrete {
        label="Domain types (one per domain)";
        style=dashed; color="#999999";
        fontname="sans-serif";

        SharedFold [label="shared::Fold<N,H,R>\nArc closures", fillcolor="#d4edda"];
        SharedTree [label="shared::Treeish<N>\nArc closures", fillcolor="#d4edda"];
        LocalFold [label="local::Fold<N,H,R>\nRc closures", fillcolor="#fff3cd"];
        OwnedFold [label="owned::Fold<N,H,R>\nBox closures", fillcolor="#f8d7da"];
        UserStruct [label="YourStruct\nimpl FoldOps\n(zero boxing)", fillcolor="#e8e8ff"];
    }

    subgraph cluster_executors {
        label="Executors (cata::exec::)";
        style=dashed; color="#999999";
        fontname="sans-serif";

        ExecDS [label="Exec<D, S>\nsole user-facing type"];
        FusedSpec [label="fused::Spec\nall domains"];
        FunnelSpec [label="funnel::Spec<P>\nShared domain\nwork-stealing"];
    }

    subgraph cluster_lifts {
        label="Lifts (prelude::)";
        style=dashed; color="#999999";
        fontname="sans-serif";

        Explainer [label="Explainer\ncomputation trace"];
    }

    FoldOps -> SharedFold [dir=back, style=dashed, label="impl"];
    FoldOps -> LocalFold [dir=back, style=dashed];
    FoldOps -> OwnedFold [dir=back, style=dashed];
    FoldOps -> UserStruct [dir=back, style=dashed];
    TreeOps -> SharedTree [dir=back, style=dashed];

    ExecDS -> FusedSpec [label="S =", style=dotted];
    ExecDS -> FunnelSpec [label="S =", style=dotted];
    ExecDS -> FoldOps [label="&impl FoldOps", style=dotted];
    ExecDS -> TreeOps [label="&impl TreeOps", style=dotted];

    LiftOps -> Explainer [dir=back, style=dashed, label="impl"];
}
```

## How a user navigates

<!-- -->

```dot process
digraph {
    rankdir=LR;
    node [shape=box, style="rounded,filled", fillcolor="#f5f5f5", fontname="sans-serif", fontsize=11];
    edge [fontname="sans-serif", fontsize=10];

    pick [label="1. Pick domain\nuse hylic::domain::shared as dom;", fillcolor="#d4edda"];
    build [label="2. Build fold + graph\ndom::simple_fold(...)\ndom::treeish(...)", fillcolor="#fff3cd"];
    run [label="3. Run\ndom::FUSED.run(&fold, &graph, &root)", fillcolor="#cce5ff"];
    par [label="3b. Or parallel\ndom::exec(funnel::Spec::default(8))\n  .run(...)", fillcolor="#cce5ff"];

    pick -> build -> run;
    build -> par [style=dashed, label="if parallel"];
}
```

Step 1 is usually `shared` and never changes. Steps 2-3 are the
entire API surface for most programs.

## Domain compatibility matrix

| | Shared | Local | Owned |
|---|:---:|:---:|:---:|
| **Fused** | yes | yes | yes |
| **Funnel** | yes | — | — |
| **Explainer** | yes | yes | — |
| **Pipeline** | yes | — | — |

Fused supports all domains (borrows, never clones). Funnel requires
`N: Clone + Send, R: Send` — the Shared domain provides these.

## Zero-boxing path

For maximum performance, skip the domain system entirely.
Implement `FoldOps` and `TreeOps` on your own structs:

```rust
struct MyFold;
impl FoldOps<MyNode, MyHeap, MyResult> for MyFold {
    fn init(&self, node: &MyNode) -> MyHeap { ... }
    fn accumulate(&self, heap: &mut MyHeap, result: &MyResult) { ... }
    fn finalize(&self, heap: &MyHeap) -> MyResult { ... }
}
```

Pass `&MyFold` directly to a Fused executor's recursion engine.
The compiler monomorphizes everything — zero vtable calls, zero
boxing, zero `Arc`. This is the absolute performance ceiling.

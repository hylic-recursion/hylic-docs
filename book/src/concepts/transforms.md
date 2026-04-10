# Transformations and Lifts

hylic's types are designed for compositional transformation. Folds
can be mapped, contramapped, zipped, and wrapped. Graphs can be
filtered, contramapped, and treemapped. These operations produce new
values from existing ones without modifying the originals (for
Clone domains) or by consuming them (for Owned).

See the [Fold guide](../guides/fold.md) and
[Graph guide](../guides/graph.md) for the full transformation API.

## Fold transformations

The fold transformation diagram summarizes what's available:

```dot process
digraph {
    rankdir=TB;
    node [shape=box, style="rounded,filled", fillcolor="#f5f5f5", fontname="monospace", fontsize=11];
    edge [fontname="sans-serif", fontsize=10];

    F [label="Fold<N, H, R>"];
    F1 [label="map\nFold<N, H, NewR>"];
    F2 [label="contramap\nFold<NewN, H, R>"];
    F3 [label="zipmap\nFold<N, H, (R, Extra)>"];
    F4 [label="wrap_init / wrap_accumulate / wrap_finalize\nFold<N, H, R> (same types, different behavior)"];
    F5 [label="product\nFold<N, (H1,H2), (R1,R2)>"];

    F -> F1; F -> F2; F -> F3; F -> F4; F -> F5;
}
```

These are all type-level transformations that compose. The fold's
three-phase structure (init/accumulate/finalize) is preserved.

## Lifts — type-domain transformations

A lift goes further than fold transformations: it transforms BOTH
the fold AND the treeish into a different type domain, runs the
computation there, and maps the result back. The caller receives
the same R — the lift is transparent.

The `LiftOps` trait defines four operations:

- **lift_treeish**: `Treeish<N>` → `Treeish<N2>`
- **lift_fold\<H\>**: `Fold<N, H, R>` → `Fold<N2, LiftedH<H>, LiftedR<H>>`
- **lift_root**: `&N` → `N2`
- **unwrap\<H\>**: `LiftedR<H>` → `R`

The lifted heap and result types are GATs on the trait, determined
by each lift implementation:

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
        F2 [label="Fold<N2, LiftedH<H>, LiftedR<H>>"];
        T2 [label="Treeish<N2>"];
    }

    R2 [label="LiftedR<H>", shape=ellipse];
    R  [label="R", shape=ellipse, style=filled, fillcolor="#d4edda"];

    F -> F2 [label="lift_fold<H>"];
    T -> T2 [label="lift_treeish"];
    F2 -> R2 [label="exec.run"];
    R2 -> R  [label="unwrap<H>"];
}
```

Execution uses `cata::lift::run_lifted`, which applies the four
transformations and runs the result through a Shared-domain executor.
H is inferred from the fold at the call site.

## Explainer — computation tracing

The `Explainer` is a unit struct implementing `LiftOps`. It wraps
the fold to record every accumulation step. The heap becomes
`ExplainerHeap` (initial state, node, transitions). The result
becomes `ExplainerResult` (original result + full trace).

```rust
{{#include ../../../src/docs_examples.rs:explainer_usage}}
```

In recursion-scheme terms, this is a histomorphism — each node
sees its subtree's full computation history.

## The mathematical picture

A fold is an F-algebra: a function `F<R> → R` that collapses one
layer of structure. hylic decomposes it into three phases
(init/accumulate/finalize) through the intermediate heap type H.

A lift is a natural transformation between two F-algebras. It maps
the carrier types through the `LiftedH` and `LiftedR` GATs while
preserving the fold structure. The `unwrap` function projects back
to R. The computation produces the same result regardless of which
algebra it runs in — the lift is transparent.

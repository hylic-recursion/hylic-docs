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
the fold AND the treeish into a different type domain. The executor
runs the lifted computation and returns `MapR<H, R>` — the caller
extracts the original result as appropriate for the lift (e.g.
`ExplainerResult::orig_result`, or identity when `MapR<H, R> = R`).

The `Lift` trait defines three operations:

- **lift_treeish**: `Treeish<N>` → `Treeish<N2>`
- **lift_fold\<H, R\>**: `Fold<N, H, R>` → `Fold<N2, MapH<H, R>, MapR<H, R>>`
- **lift_root**: `&N` → `N2`

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
        F2 [label="Fold<N2, MapH<H, R>, MapR<H, R>>"];
        T2 [label="Treeish<N2>"];
    }

    R2 [label="MapR<H, R>", shape=ellipse, style=filled, fillcolor="#fff3cd"];

    F -> F2 [label="lift_fold<H, R>"];
    T -> T2 [label="lift_treeish"];
    F2 -> R2 [label="exec.run"];
}
```

`cata::lift::run_lifted` applies the three transformations, runs the
lifted computation through a Shared-domain executor, and returns
`MapR<H, R>`. H and R are inferred from the fold at the call site.

## Explainer — computation tracing

The `Explainer` is a unit struct implementing `Lift`. It wraps
the fold to record every accumulation step. The heap becomes
`ExplainerHeap` (initial state, node, transitions). The result
becomes `ExplainerResult` (original result + full trace).

```rust
{{#include ../../../src/docs_examples.rs:explainer_usage}}
```

In recursion-scheme terms, this is a histomorphism — each node
sees its subtree's full computation history.

## The mathematical picture

The catamorphism's algebra is `F R → R` — collapse one layer with
children already folded to R. hylic factors this through a working
type `H`: init creates `H` from the node, accumulate folds child
results `R` into `H`, finalize projects `H → R`. The carrier is `R`
at every subtree. `H` is internal to the bracket. See
[The N-H-R algebra factorization](../design/milewski.md) for the
correspondence with Milewski's monoidal decomposition and the
equivalence conditions.

A lift is an algebra morphism: it maps the carrier types through
`MapH` and `MapR` while preserving the fold structure.

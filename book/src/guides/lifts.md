# Lifts: cross-cutting concerns

A lift transforms both the fold and the treeish into a different type
domain. The executor runs the lifted computation and returns
`LiftedR<H>` — the caller extracts the original result as needed.

The `LiftOps` trait defines three operations. The lifted heap and
result types are GATs (`LiftedH<H>`, `LiftedR<H>`), so each lift
determines how H maps to its lifted counterpart without requiring H
as a trait-level parameter:

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

    R2 [label="LiftedR<H>", shape=ellipse, style=filled, fillcolor="#fff3cd"];

    F -> F2 [label="lift_fold<H>"];
    T -> T2 [label="lift_treeish"];
    F2 -> R2 [label="exec.run"];
}
```

## Explainer — computation tracing

The `Explainer` is a unit struct implementing `LiftOps<N, R, N>` for
all Clone types. It records every step of the fold at every node:
the initial heap, each child result accumulated, and the final result.
This is a histomorphism — each node sees its subtree's full
computation history.

```rust
{{#include ../../../src/docs_examples.rs:explainer_usage}}
```

The `ExplainerResult` contains the original result (`.orig_result`)
plus the full `ExplainerHeap` — initial state, node, transitions,
and working heap.

## The LiftOps trait

A lift provides three operations:

- **lift_treeish**: transform `Treeish<N>` → `Treeish<N2>`
- **lift_fold\<H\>**: transform `Fold<N, H, R>` → `Fold<N2, LiftedH<H>, LiftedR<H>>`
- **lift_root**: transform `&N` → `N2`

`lift_fold` is generic over H (the original fold's heap type). The
trait's GATs `LiftedH<H>` and `LiftedR<H>` determine the lifted
types per lift implementation. H is bounded by `Clone + 'static` —
lifts inherently copy heap state between phases.

Concrete lifts implement `LiftOps` directly as structs. The Explainer
is a unit struct (no state). The `SeedLift` (used internally by
[`SeedPipeline`](./seed_pipeline.md)) carries a grow function.
Parallel lifts in the `hylic-parallel-lifts` crate carry pool
references. See [Implementing a custom lift](./implementing_lifts.md)
for the step-by-step pattern.

## Execution

`cata::lift::run_lifted` applies the three transformations and runs
the result through any Shared-domain executor:

```rust
use hylic::cata::lift;

// Returns ExplainerResult — access .orig_result for R
let trace = lift::run_lifted(&exec, &Explainer, &fold, &graph, &root);
```

H is inferred from the fold. The executor runs the lifted fold on the
lifted treeish and returns `LiftedR<H>`.

For the theoretical basis (algebra morphisms, how lifts relate to
Milewski's monoidal decomposition), see
[The N-H-R algebra factorization](../design/milewski.md).

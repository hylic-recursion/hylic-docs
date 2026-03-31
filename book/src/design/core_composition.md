# Core composition

hylic decomposes recursive tree computation into independently
definable, independently transformable pieces. This page shows
how they compose.

## The pieces

Three independent definitions compose into a result:

```dot process
digraph {
    rankdir=LR;
    node [shape=box, style="rounded,filled", fillcolor="#f5f5f5", fontname="monospace", fontsize=11];
    edge [fontname="sans-serif", fontsize=10];

    Fold    [label="Fold<N, H, R>\ninit / accumulate / finalize"];
    Treeish [label="Treeish<N>\ngiven a node, visit children"];
    Exec    [label="Exec<N, R>\nfused / sequential / rayon"];
    Result  [label="R"];

    Fold -> Exec [label="algebra"];
    Treeish -> Exec [label="structure"];
    Exec -> Result [label="run"];
}
```

**Fold** defines what to compute at each node: initialize a heap,
fold each child's result into it, finalize the heap into the node's
result. It knows nothing about tree structure.

**Treeish** defines the tree: given a node, call a callback for each
child. It knows nothing about what is computed. Callback-based
traversal means zero allocation per node.

**Exec** drives the execution. `Exec::fused()` recurses via callbacks
(zero allocation). `Exec::rayon()` parallelizes sibling subtrees.
The executor is parameterized by a child-visiting lambda — the
lambda encapsulates the traversal mode and any parallelism bounds.

## Transformations

Because Fold is data (three closures behind Arc), you transform it
rather than rewrite it:

```dot process
digraph {
    rankdir=LR;
    node [shape=box, style="rounded,filled", fillcolor="#f5f5f5", fontname="monospace", fontsize=11];
    edge [fontname="sans-serif", fontsize=10];

    F1 [label="Fold<N, H, R>"];
    F2 [label="Fold<N, H, R>\nwith logging"];
    F3 [label="Fold<N, H, (R, Extra)>"];
    F4 [label="Fold<NewN, H, R>"];
    F5 [label="Fold<N, (H1,H2), (R1,R2)>"];

    F1 -> F2 [label="map_init"];
    F1 -> F3 [label="zipmap"];
    F1 -> F4 [label="contramap"];
    F1 -> F5 [label="product"];
}
```

- **map_init / map_accumulate / map_finalize** — wrap individual phases.
- **map** — change the result type R → R' (with backmapper).
- **zipmap** — augment R with derived data: R → (R, Extra).
- **contramap** — change the node type: Fold<N,...> → Fold<NewN,...>.
- **product** — two folds in one pass: (R1, R2) from one traversal.

Similarly, Treeish/Edgy has: **map**, **contramap**, **contramap_or**,
**filter**, **treemap**. Graph has **map_treeish**, **map_top_edgy**.

## The layers

Each layer only depends downward:

```dot process
digraph {
    rankdir=LR;
    node [shape=box, style="rounded,filled", fillcolor="#f5f5f5", fontname="monospace", fontsize=11];
    edge [fontname="sans-serif", fontsize=10];

    uio     [label="uio\nlazy computation (FnOnce)"];
    graph_  [label="graph\nEdgy, Treeish, Graph\nVisit, SeedGraph"];
    fold    [label="fold\nFold, init/accumulate/finalize"];
    cata    [label="cata\nExec, Lift"];
    pipe    [label="pipeline\nGraphWithFold"];
    prelude [label="prelude\nVecFold, Explainer, memoize\nseeds_for_fallible, uio_parallel"];

    uio -> cata;
    graph_ -> cata;
    fold -> cata;
    graph_ -> pipe;
    cata -> pipe;
    fold -> pipe;
    graph_ -> prelude;
    fold -> prelude;
    cata -> prelude;
    uio -> prelude;
}
```

`graph` and `fold` are independent of each other. `cata` combines
them via `Exec` and provides `Lift` for type-domain transformations.
`pipeline` wires graph + fold + entry point into `GraphWithFold`.
`prelude` provides batteries: VecFold, Explainer, memoization,
fallible seed helpers, and UIO-based parallelization.

## SeedGraph (in graph/)

`SeedGraph<Node, Seed, Top>` defines how to unfold a tree from seeds:
- **seeds_from_node**: given a node, what are its dependency seeds?
- **grow**: given a seed, produce a node
- **seeds_from_top**: entry point → initial seeds

It's general — no assumption about the Node type. For fallible
resolution (Either<Error, Valid> nodes), the prelude provides
`seeds_for_fallible` which lifts a valid-only seed function so
errors become leaves.

## Lift (in cata/)

`Lift<N,H,R, N2,H2,R2>` is a paired transformation that lifts
Treeish + Fold to another type domain. The lifted computation runs
internally (via `Exec::run_lifted`); unwrap recovers the original
result type.

The library's UIO-based parallelization (`uio_parallel()`) is a Lift:
it transforms the fold to produce `UIO<R>` results where sibling
subtrees evaluate in parallel. The Treeish is unchanged — parallelism
is purely in the fold's result domain.

This is the pattern described in the next section,
[The two-function problem](./two_function_problem.md).

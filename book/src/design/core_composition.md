# Core composition

hylic decomposes recursive tree computation into independently
definable, independently transformable pieces. This page shows
how they compose.

## The pieces

```mermaid
graph TD
    F["Fold&lt;N, H, R&gt;<br/>init / accumulate / finalize"]
    G["Treeish&lt;N&gt;<br/>given a node, visit children"]
    S["Strategy<br/>Sequential / ParTraverse / ParFoldLazy"]
    R["Result R"]

    F --> |algebra| S
    G --> |structure| S
    S --> |execute| R
```

**Fold** defines what to compute at each node: initialize a heap,
fold each child's result into it, finalize the heap into the node's
result. It knows nothing about tree structure.

**Treeish** defines the tree: given a node, call a callback for each
child. It knows nothing about what is computed. Callback-based
traversal means zero allocation per node.

**Strategy** drives the execution: walk the tree (via Treeish),
apply the fold at each node (via Fold), return the root's result.
Sequential does this in a single recursion. Parallel strategies
fan out sibling subtrees via rayon.

## Transformations

Because Fold is data (three closures behind Arc), you transform it
rather than rewrite it:

```mermaid
graph LR
    F1["Fold&lt;N, H, R&gt;"]
    F2["Fold&lt;N, H, R&gt;<br/>(with logging)"]
    F3["Fold&lt;N, H, (R, Extra)&gt;"]

    F1 --> |map_init| F2
    F1 --> |zipmap| F3
```

- **map_init / map_accumulate / map_finalize** — wrap individual phases.
  Logging, validation, side effects.
- **map** — change the result type R → R'. Requires a backmapper
  for children's results to flow through accumulate.
- **zipmap** — augment R with derived data: R → (R, Extra).
  The Extra is per-node (derived from that node's R), not accumulated.

The same Treeish can be used with different Folds. The same Fold
can run over different trees. Transformations compose without
touching either.

## The layers

```mermaid
graph BT
    U["uio — lazy memoized computation"]
    UT["utils — string helpers"]
    G["graph — Edgy, Treeish, Graph, Visit"]
    F["fold — Fold, init/accumulate/finalize"]
    C["cata — Strategy, sync/parallel execution"]
    A["ana — SeedGraph, error-handling builders"]
    H["hylo — FoldAdapter, SeedFoldAdapter,<br/>GraphWithFold, SeedGraphFold"]
    P["prelude — VecFold, Explainer,<br/>TreeFormatCfg, memoize, common folds"]

    U --> G
    U --> C
    UT --> F
    G --> C
    F --> C
    G --> A
    A --> H
    C --> H
    F --> H
    G --> P
    F --> P
    C --> P
    U --> P
```

Each layer only depends downward. `graph` and `fold` are independent
of each other. `cata` combines them. `ana` builds graphs from seeds.
`hylo` wires everything into execution adapters. `prelude` provides
convenience types built on all of the above.

## Seed-based graphs (ana)

The `ana` module is not core in the same way `fold` and `graph` are.
It's a construction pattern built entirely on the core mechanisms,
showing how to bridge the gap between a starting "seed" and a full
recursive tree.

`SeedGraph` defines three things:
- How to get dependency seeds from a resolved node
- How to grow a seed into a resolved node (or an error)
- How to get the initial seeds from a top-level entry point

From these, it constructs a `Treeish` and a `Graph` — standard core
types. The `hylo` module then wires this graph with a `Fold` for
execution.

This is the pattern described in the next section,
[The two-function problem](./two_function_problem.md).

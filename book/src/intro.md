# hylic

A Rust library for composable recursive tree computation.

hylic separates a recursive computation into three independent
concerns: a **fold** that defines what to compute at each node, a
**graph** that describes the tree structure, and an **executor** that
controls how the recursion is carried out. Each concern can be
defined, transformed, and composed independently.

```rust
{{#include ../../src/docs_examples.rs:intro_dir_example}}
```

The tree structure need not live inside the data. A `Treeish` is a
function from a node to its children — it can traverse a nested
struct, look up indices in a flat array, or resolve references
through any external mechanism:

```rust
{{#include ../../src/docs_examples.rs:intro_flat_example}}
```

## Architecture

User-defined closures are wrapped into composable types (Fold,
Treeish), transformed independently, and handed to an executor. The
executor drives a recursion where fold and graph interleave at every
node:

```dot process
digraph hylic {
    rankdir=TB;
    compound=true;
    newrank=true;
    node [fontname="sans-serif", fontsize=10, shape=box, style="rounded,filled"];
    edge [fontname="sans-serif", fontsize=9];

    subgraph cluster_fold_def {
        label=""; style=invis;
        init [label="init: &N → H", fillcolor="#d4edda"];
        acc  [label="accumulate: &mut H, &R", fillcolor="#d4edda"];
        fin  [label="finalize: &H → R", fillcolor="#d4edda"];
        init -> acc -> fin [style=invis];
    }

    subgraph cluster_graph_def {
        label=""; style=invis;
        visit [label="visit: &N → children", fillcolor="#cce5ff"];
    }

    fold    [label="Fold<N, H, R>", fillcolor="#a5d6a7", penwidth=2];
    treeish [label="Treeish<N>", fillcolor="#90caf9", penwidth=2];

    init -> fold;
    acc  -> fold;
    fin  -> fold;
    visit -> treeish;

    fold_t [label="map · zipmap · contramap\nproduct · wrap_*", fillcolor="#e8f5e9", fontsize=9, shape=note];
    tree_t [label="filter · contramap\nmemoize", fillcolor="#e3f2fd", fontsize=9, shape=note];

    fold -> fold_t [style=dashed, arrowhead=none];
    fold_t -> fold [style=dashed, label="new Fold"];
    treeish -> tree_t [style=dashed, arrowhead=none];
    tree_t -> treeish [style=dashed, label="new Treeish"];

    exec [label="exec.run(&fold, &graph, &root) → R", fillcolor="#fff59d", penwidth=2, fontsize=11];

    fold -> exec [penwidth=1.5];
    treeish -> exec [penwidth=1.5];

    subgraph cluster_recurse {
        label="at each node"; labeljust=l;
        style="rounded,filled"; fillcolor="#fafafa"; color=grey80;
        fontname="sans-serif"; fontsize=10;

        s1 [label="① fold.init(&node) → heap", fillcolor="#c8e6c9"];
        s2 [label="② graph.visit(&node, |child| …)", fillcolor="#bbdefb"];
        s3 [label="③ fold.accumulate(&mut heap, &child_r)", fillcolor="#c8e6c9"];
        s4 [label="④ fold.finalize(&heap) → R", fillcolor="#c8e6c9"];
        s1 -> s2 -> s3 -> s4;
        s3 -> s2 [label="per child", style=dashed, constraint=false];
    }

    exec -> s1 [lhead=cluster_recurse];

    fused  [label="Fused\ndirect recursion\nany domain, any graph", fillcolor="#fff3cd"];
    funnel [label="Funnel<P>\nCPS work-stealing\nthree monomorphized policy axes", fillcolor="#ffe0b2"];

    {rank=same; fused; funnel}
    s4 -> fused [style=invis];
    s4 -> funnel [style=invis];
}
```

- **`N`** — the node type (a struct, an index, a key — anything)
- **`H`** — the heap: per-node mutable scratch space, created by `init`, not shared between nodes
- **`R`** — the result: produced by `finalize`, flows upward to the parent's `accumulate`

Any fold and graph can be executed in parallel by switching to the
[Funnel executor](./funnel/overview.md) — a
[CPS work-stealing](./funnel/cps_walk.md) engine where unfold and fold
interleave without materializing the tree. Three
[compile-time policy axes](./funnel/policies.md) control
[queue topology](./funnel/queue_strategies.md),
[accumulation strategy](./funnel/accumulation.md), and
[wake policy](./funnel/pool_dispatch.md), all monomorphized to zero
dispatch overhead. Child results flow back through a
[packed-ticket FoldChain](./funnel/cascade.md) with
[destructive streaming sweeps](./funnel/accumulation.md) that free
intermediate memory progressively. See
[Benchmarks](./cookbook/benchmarks.md) for the performance
characteristics.

## Transformations and lifts

Folds and graphs are independently transformable. Each combinator
produces a new value — the original is unchanged (for Clone domains)
or consumed (for Owned):

```dot process
digraph transforms {
    rankdir=TB;
    compound=true;
    node [fontname="monospace", fontsize=9, shape=box, style="rounded,filled"];
    edge [fontname="sans-serif", fontsize=9];

    subgraph cluster_fold {
        label="Fold<N, H, R>"; labeljust=l;
        style="rounded,filled"; fillcolor="#f1f8e9"; color="#a5d6a7";
        fontname="sans-serif"; fontsize=10;

        fmap    [label=".map(Fn(&R)→RNew, Fn(&RNew)→R)\n→ Fold<N, H, RNew>", fillcolor="#c8e6c9"];
        fzip    [label=".zipmap(Fn(&R)→Extra)\n→ Fold<N, H, (R, Extra)>", fillcolor="#c8e6c9"];
        fcontra [label=".contramap(Fn(&NewN)→N)\n→ Fold<NewN, H, R>", fillcolor="#c8e6c9"];
        fprod   [label=".product(&Fold<N, H2, R2>)\n→ Fold<N, (H,H2), (R,R2)>", fillcolor="#c8e6c9"];
        fwrap   [label=".wrap_init(Fn(&N, &dyn Fn(&N)→H)→H)\n.wrap_accumulate(Fn(&mut H, &R, &dyn Fn(&mut H, &R)))\n.wrap_finalize(Fn(&H, &dyn Fn(&H)→R)→R)", fillcolor="#c8e6c9"];
        fmap -> fzip -> fcontra -> fprod -> fwrap [style=invis];
    }

    subgraph cluster_graph {
        label="Treeish<N>"; labeljust=l;
        style="rounded,filled"; fillcolor="#e3f2fd"; color="#90caf9";
        fontname="sans-serif"; fontsize=10;

        gfilter [label=".filter(Fn(&N)→bool)\n→ Treeish<N>", fillcolor="#bbdefb"];
        gmemo   [label="memoize_treeish(&Treeish<N>)\n→ Treeish<N>  (cached for DAGs)", fillcolor="#bbdefb"];
        gcontra [label=".contramap(Fn(&NewN)→N)\n→ Treeish<NewN>", fillcolor="#bbdefb"];
        gtmap   [label=".treemap(Fn(&N)→NewN, Fn(&NewN)→N)\n→ Treeish<NewN>", fillcolor="#bbdefb"];
        gfilter -> gmemo -> gcontra -> gtmap [style=invis];
    }
}
```

- **`N`, `NewN`** — original and target node types
- **`H`** — the fold's per-node heap (unchanged by map/zipmap/contramap)
- **`R`, `RNew`, `Extra`** — original, replaced, and augmented result types

All compose freely — see the
[Fold guide](./guides/fold.md), [Graph guide](./guides/graph.md),
and [Transformations cookbook](./cookbook/transformations.md).

A [lift](./guides/lifts.md) goes further — it transforms both fold
and treeish in sync into a different type domain via the
[`LiftOps`](./concepts/transforms.md) trait. The
[Explainer](./concepts/transforms.md#explainer--computation-tracing)
records the full computation trace at every node (histomorphism).

[`SeedPipeline`](./guides/seed_pipeline.md) handles a common case:
the tree is discovered lazily from *seed* references rather than
known upfront. The user provides a seed edge function
(`Edgy<N, Seed>`) and a `grow` function (`Fn(&Seed) -> N`); the
pipeline constructs the treeish, handles the entry transition, and
runs the fold. Internally it uses a lift (`SeedLift`), but the
`LiftedNode<Seed, N>` type is hidden entirely.

## Cookbook

The [Cookbook](./cookbook/fibonacci.md) contains worked examples with
snapshot-tested output:
[expression evaluation](./cookbook/expression_eval.md),
[module resolution](./cookbook/module_resolution.md),
[configuration inheritance](./cookbook/config_inheritance.md),
[filesystem summary](./cookbook/filesystem_summary.md),
[cycle detection](./cookbook/cycle_detection.md),
[parallel execution](./cookbook/parallel_execution.md).

## Where to start

The [Quick Start](./quickstart.md) walks through constructing and
running a fold. [The recursive pattern](./concepts/separation.md)
explains the underlying decomposition.

## Further reading

- Meijer, Fokkinga, Paterson. *Functional Programming with Bananas, Lenses, Envelopes and Barbed Wire.* (1991) — the original recursion schemes paper.
- Milewski. [Monoidal Catamorphisms](https://bartoszmilewski.com/2020/06/15/monoidal-catamorphisms/) (2020) — a different algebra factorization. See [comparison](./design/milewski.md).
- Kmett. [recursion-schemes](https://hackage.haskell.org/package/recursion-schemes) — Haskell reference implementation.
- Malick. [recursion.wtf](https://recursion.wtf/) — practical recursion schemes in Rust.

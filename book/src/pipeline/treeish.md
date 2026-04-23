# Stage 1 — TreeishPipeline

`TreeishPipeline` is the lighter Stage-1 variant for when you
don't need the Seed/Grow machinery. Just two slots: a treeish
and a fold.

```rust
{{#include ../../../../hylic-pipeline/src/treeish/mod.rs:treeish_pipeline_struct}}
```

- **`treeish: Graph<N>`** — `N → N*`, direct child enumeration.
- **`fold: Fold<N, H, R>`** — the algebra.

No grow step, no entry seeds. You run it by handing a starting
`&N` to the executor.

## When to pick this

When your data already gives you `N → children: &[N]`. Examples:

- An AST where each node has `children: Vec<Node>`.
- A filesystem-like tree held in memory.
- Any structure where the children type equals the node type.

If you need to resolve references (`Seed → N`), use
[SeedPipeline](./seed.md).

## Constructing one

```rust
use hylic_pipeline::prelude::*;

#[derive(Clone)]
struct Node { value: u64, children: Vec<Node> }

let tp: TreeishPipeline<Shared, Node, u64, u64> =
    TreeishPipeline::new(
        treeish(|n: &Node| n.children.clone()),  // N -> N*
        &fold(
            |n: &Node| n.value,
            |h: &mut u64, c: &u64| *h += c,
            |h: &u64| *h,
        ),
    );
```

## Stage-1 reshape (inherent via trait)

Just one sugar at Stage 1:

| method                     | changes                          |
|----------------------------|----------------------------------|
| `map_node_bi(co, contra)`  | changes N via bijection          |

Provided by the [`TreeishSugarsShared`](./sugars.md) blanket
trait (or `TreeishSugarsLocal` for the Local domain).

## Stage-2 sugars via auto-lift

Like SeedPipeline, Stage-2 sugars work directly:

```rust
let r: u64 = tp
    .wrap_init(|n, orig| orig(n) + 1)
    .zipmap(|r| *r > 100)
    .run_from_node(&FUSED, &root_node);
```

`.wrap_init(...)` auto-lifts the TreeishPipeline and composes the
sugar.

## Running it

Via `PipelineExec::run_from_node`:

```rust
let r: u64 = tp.run_from_node(&FUSED, &root_node);
```

No entry heap — unlike SeedPipeline, there's no synthetic Entry
level to initialise. The first `init` call happens at `root_node`.

## Relation to bare lift

`TreeishPipeline::new(treeish, &fold)` stores essentially the
same two things as `LiftBare::apply_bare` operates on. The
difference is that TreeishPipeline is a Stage-1 typestate —
transforms produce typed TreeishPipelines — while bare usage is
just closures.

Pick the pipeline when you want the fluent builder; pick bare when
you have a lift you want to apply one-shot.

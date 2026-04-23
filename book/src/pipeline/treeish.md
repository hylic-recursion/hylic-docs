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

When your data already has `N → children: &[N]`. Examples:

- An AST where each node has `children: Vec<Node>`.
- A filesystem-like tree held in memory.
- Any structure where the children type equals the node type.

If you need to resolve references (`Seed → N`), use
[SeedPipeline](./seed.md).

## Constructing one

```rust
{{#include ../../../src/docs_examples.rs:treeish_pipeline_ctor}}
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
{{#include ../../../src/docs_examples.rs:treeish_pipeline_chain}}
```

`.wrap_init(...)` auto-lifts the TreeishPipeline and composes the
sugar. The `r` return is `(u64, bool)` — `.zipmap` changed the R
axis to carry both the original sum and the boolean derivative;
`.run_from_node(...)` hands back the tip R verbatim.

## Running it

Via `PipelineExec::run_from_node(&exec, &root)` — the return type
reflects the tip R of the lift chain (or the base Fold's R, if no
lifts are composed).

No entry heap — unlike SeedPipeline, there's no synthetic Entry
level to initialise. The first `init` call happens at `root_node`.

## Relation to bare lift

`TreeishPipeline::new(treeish, &fold)` stores the same two things
`LiftBare::apply_bare` operates on. The difference: TreeishPipeline
is a Stage-1 typestate, so transforms produce typed
TreeishPipelines and chain methods work. Bare usage has neither.

Pick the pipeline when you want chained transforms; pick bare when
you have a lift to apply directly.

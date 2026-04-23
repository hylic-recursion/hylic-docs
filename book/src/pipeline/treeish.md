# Stage 1 — TreeishPipeline

`TreeishPipeline` is the lighter Stage-1 variant, appropriate
when the Seed/Grow machinery is not required. It carries only
two slots: a treeish and a fold.

```rust
{{#include ../../../../hylic-pipeline/src/treeish/mod.rs:treeish_pipeline_struct}}
```

- **`treeish: Graph<N>`** — `N → N*`, direct child enumeration.
- **`fold: Fold<N, H, R>`** — the algebra.

No grow step and no entry seeds: execution is initiated by
supplying a starting `&N` to the executor.

## When to pick this

Use `TreeishPipeline` when the nodes themselves enumerate
children of the same type — `N → children: &[N]`. Typical cases:

- an AST in which each node has `children: Vec<Node>`;
- a filesystem-like tree held in memory;
- any structure in which the child type coincides with the node
  type.

When references must be resolved before the children become
visible (`Seed → N`), [SeedPipeline](./seed.md) is appropriate.

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

As with `SeedPipeline`, Stage-2 sugars may be applied directly:

```rust
{{#include ../../../src/docs_examples.rs:treeish_pipeline_chain}}
```

`.wrap_init(...)` auto-lifts the `TreeishPipeline` and composes
the sugar. The return `r` is `(u64, bool)`: `.zipmap` extended
the R axis to carry both the original sum and the boolean
derivative, and `.run_from_node(...)` returns the tip R
unmodified.

## Running it

Execution is performed through `PipelineExec::run_from_node(&exec, &root)`.
The return type reflects the tip `R` of the lift chain, or the
base fold's `R` when no lifts are composed.

There is no entry heap: unlike `SeedPipeline`, the Treeish
variant has no synthetic Entry level to initialise. The first
`init` occurs at the supplied `root_node`.

## Relation to bare lift application

`TreeishPipeline::new(treeish, &fold)` stores the same two
objects that `LiftBare::apply_bare` operates on. The distinction
lies in typing: `TreeishPipeline` is a Stage-1 typestate, so
transforms produce typed `TreeishPipelines` and chain methods
remain available. Bare usage lacks both.

The pipeline form is appropriate where chained transforms are
needed; the bare form is appropriate where a single lift is to
be applied directly.

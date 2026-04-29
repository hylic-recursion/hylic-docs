# One-shot — `OwnedPipeline`

```rust
{{#include ../../../../hylic-pipeline/src/owned/mod.rs:owned_pipeline_struct}}
```

Two slots, like `TreeishPipeline`, but stored in the `Owned`
domain — closures are `Box<dyn Fn>`, not `Clone`, not
`Send + Sync`. Runs once and is consumed.

## Constructor

```rust
let pipeline = OwnedPipeline::new(
    treeish,    // owned::Edgy<N, N>
    fold,       // owned::Fold<N, H, R>
);
```

## Running

```rust
let r = pipeline.run_from_node_once(&FUSED, &root);
// pipeline is consumed.
```

`run_from_node_once` is the by-value method on
`PipelineExecOnce`, the consuming counterpart of
`PipelineExec::run_from_node`. `Owned` does not implement
`ShapeCapable`, so Stage-2 sugars are not available — there is
no chain to compose.

## Worked example

```rust
{{#include ../../../src/docs_examples.rs:owned_pipeline_example}}
```

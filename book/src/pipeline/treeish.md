# Stage 1 — `TreeishPipeline`

```rust
{{#include ../../../../hylic-pipeline/src/treeish/mod.rs:treeish_pipeline_struct}}
```

Two slots:

- **`treeish: <D as Domain<N>>::Graph<N>`** — direct child
  enumeration, `N → N*`.
- **`fold: <D as Domain<N>>::Fold<H, R>`** — the algebra over
  `N`.

No grow step, no entry seeds. Execution starts from a `&N`
root supplied to the executor.

## Constructors

```rust
// Shared domain.
TreeishPipeline::<Shared, _, _, _>::new(
    treeish_arc,         // hylic::graph::Treeish<N>
    &fold,               // &shared::Fold<N, H, R>
);

// Local domain — note the `_local` suffix; Rust's inherent-method
// resolution can't disambiguate two `new`s on the same struct that
// differ only in the domain marker.
TreeishPipeline::<Local, _, _, _>::new_local(
    treeish_local,       // local::Edgy<N, N>
    fold_local,          // local::Fold<N, H, R>
);

// Domain-generic.
TreeishPipeline::<D, _, _, _>::from_slots(treeish, fold);
```

## Stage-1 reshape

One sugar — there's no `grow` axis to reshape and no seeds to
filter:

| method                    | output                            |
|---------------------------|-----------------------------------|
| `map_node_bi(co, contra)` | `TreeishPipeline<D, N2, H, R>`    |

Provided by `TreeishSugarsShared` (Local mirror:
`TreeishSugarsLocal`); see [Sugars](./sugars.md).

## Stage 2

Two ways to enter:

- Explicit: `tree_pipeline.lift()` returns
  `Stage2Pipeline<TreeishPipeline<D, N, H, R>, IdentityLift>`.
- Auto-lift: every Stage-2 sugar is also callable directly on
  `TreeishPipeline`. `tree_pipeline.wrap_init(w)` is shorthand
  for `tree_pipeline.lift().wrap_init(w)`.

```rust
{{#include ../../../src/docs_examples.rs:treeish_pipeline_chain}}
```

The chain's input N stays at the user's `N` (no wrap layer);
the [`Wrap`](./wrap_dispatch.md) impl is `Identity`.

## Running

```rust
let r = pipeline.run_from_node(&FUSED, &root);
```

`PipelineExec::run_from_node(&exec, &root)` is a blanket method
on every `TreeishSource`. The first `init` runs on the supplied
`root`. Returns the chain-tip `R` — the base fold's `R` when no
Stage-2 sugars are composed, otherwise whatever the rightmost
lift produces.

`Stage2Pipeline<TreeishPipeline<…>, L>` inherits the same
method through its `TreeishSource` impl; the call shape is
identical.

## Worked example

```rust
{{#include ../../../src/docs_examples.rs:treeish_pipeline_ctor}}
```

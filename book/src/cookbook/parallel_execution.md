# Parallel execution

Same fold with both strategies — identical results. Strategy
choice is independent of fold and graph definition.

> **Imports:** `hylic::fold::simple_fold`, `hylic::graph::treeish`, `hylic::cata::{Strategy, ALL}`
>
> **Strategies** (in `hylic::cata`): `Sequential`, `Par`
>
> **Composed executor** (in `hylic::cata`): `Par<N, H, R>` — with `map_fold`, `map_plan_transform`, `build`

```rust
{{#include ../../../src/cookbook/parallel_execution.rs}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__parallel_execution__tests__parallel.snap:5:}}
```

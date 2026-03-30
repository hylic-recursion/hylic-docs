# Module resolution

The SeedGraph pattern: unfold from seeds, handle errors as
`Either::Left` nodes, fold results bottom-up.

> **Imports:**
> - `hylic::ana::SeedGraph` — anamorphism: `seeds_from_top`, `grow_node`, `seeds_from_valid`
> - `hylic::hylo::SeedFoldAdapter` — wires SeedGraph + Fold into a runnable pipeline
> - `hylic::graph::edgy` — edge constructor for seed graphs
> - `either::Either` — `Left(error)` or `Right(valid)` node types

```rust
{{#include ../../../src/cookbook/module_resolution.rs}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__module_resolution__tests__resolution.snap:5:}}
```

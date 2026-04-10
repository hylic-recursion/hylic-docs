# Module resolution

The seed-based graph pattern for lazy tree discovery. A `grow`
function resolves dependency references (seeds) into modules (nodes),
which may themselves have dependencies. Error handling uses
`Either<Error, Valid>` — error nodes are leaves with no children.

This example uses `SeedGraph` and `GraphWithFold` to build and run
the resolution pipeline. See [Entry points](../concepts/entry.md)
for the lift-based `SeedPipeline` alternative, which makes the
seed layer explicit and composable.

```rust
{{#include ../../../src/cookbook/module_resolution.rs}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__module_resolution__tests__resolution.snap:5:}}
```

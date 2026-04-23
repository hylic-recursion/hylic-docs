# Module resolution

Lazy dependency resolution via `SeedPipeline`. A `grow` function
resolves dependency references (seeds) into modules (nodes), which
may themselves have dependencies. Error handling uses
`Either<Error, Valid>` — error nodes are leaves with no children.

See [Seed-based lazy discovery](../pipeline/seed.md) for
the `SeedPipeline` API and its internal mechanics.

```rust
{{#include ../../../src/cookbook/module_resolution.rs}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__module_resolution__tests__resolution.snap:5:}}
```

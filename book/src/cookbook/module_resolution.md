# Module resolution

The pattern that motivated hylic. The entry point (top-level spec) differs
from the recursive part (module dependencies). `SeedGraph` separates these:
seeds expand into nodes, nodes expand into more seeds, errors are just
nodes with no children.

```rust
{{#include ../../../src/cookbook/module_resolution.rs:module_resolution}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__module_resolution__tests__resolution.snap:5:}}
```

# Cycle detection

Detect cycles in a dependency graph. Cycle state lives in the
node type (ancestor set), not the fold — the Treeish decides
structure, the Fold just collects.

```rust
{{#include ../../../src/cookbook/cycle_detection.rs}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__cycle_detection__tests__detect_cycles.snap:5:}}
```

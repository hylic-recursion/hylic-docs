# Cycle detection

Detect cycles in a dependency graph during traversal. Each node carries
its ancestor set — if a node appears in its own ancestors, it's a cycle.
Cycles become leaves (no further recursion).

```rust
{{#include ../../../src/cookbook/cycle_detection.rs:cycle_detection}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__cycle_detection__tests__cycles.snap:5:}}
```

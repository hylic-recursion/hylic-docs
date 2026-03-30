# Cycle detection

Detect cycles in a dependency graph during traversal. Each node
carries its ancestor set — cycles become leaves (no further recursion).

> **Imports:** `hylic::fold::simple_fold`, `hylic::graph::treeish`, `hylic::cata::Strategy`

```rust
{{#include ../../../src/cookbook/cycle_detection.rs}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__cycle_detection__tests__cycles.snap:5:}}
```

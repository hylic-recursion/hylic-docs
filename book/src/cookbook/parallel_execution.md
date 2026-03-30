# Parallel execution

Same fold, different execution strategies. All strategies produce identical
results — parallelism is a traversal concern, not an algebra concern.

```rust
{{#include ../../../src/cookbook/parallel_execution.rs:parallel_execution}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__parallel_execution__tests__parallel.snap:5:}}
```

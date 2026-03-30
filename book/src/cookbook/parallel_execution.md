# Parallel execution

Same fold, different executors — identical results. `Exec::fused()`
uses callback-based recursion (zero allocation). `Exec::rayon()`
parallelizes sibling subtrees via rayon. `Exec::sequential()`
collects children to Vec, processes one by one.

```rust
{{#include ../../../src/cookbook/parallel_execution.rs}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__parallel_execution__tests__parallel.snap:5:}}
```

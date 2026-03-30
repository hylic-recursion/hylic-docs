# Transformations

Folds are data — transform them without rewriting.

## Logging via map_init

Wrap the init phase to record which nodes are visited:

```rust
{{#include ../../../src/cookbook/transformations.rs:transformations}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__transformations__tests__logged.snap:5:}}
```

## zipmap: per-node annotations

Derive extra data from each node's result. Here, classify subtree totals:

```rust
{{#include ../../../src/cookbook/transformations.rs:zipmap}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__transformations__tests__zipmap.snap:5:}}
```

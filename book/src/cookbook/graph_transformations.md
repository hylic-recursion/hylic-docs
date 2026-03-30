# Graph transformations

A progression from simple static trees to lookup-based graphs,
selective logging, and caching — showing how hylic's pieces
compose incrementally.

## Domain model

All examples share a build system domain:

```rust
{{#include ../../../src/cookbook/graph_transformations.rs:build_domain}}
```

## 1. Simple static graph

Tree stored directly in structs — `treeish_from` gives zero-clone
access to the children field:

```rust
{{#include ../../../src/cookbook/graph_transformations.rs:simple_graph}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__graph_transformations__tests__simple_graph.snap:5:}}
```

## 2. Graph from lookup

Dependencies resolved by name — the graph is constructed lazily
from a lookup table. Same fold, different tree construction:

```rust
{{#include ../../../src/cookbook/graph_transformations.rs:lookup_graph}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__graph_transformations__tests__lookup_graph.snap:5:}}
```

## 3. Selective logging

VecFold gives finalize access to both the node and all child results.
Here we compute subtree totals and log tasks exceeding a threshold:

```rust
{{#include ../../../src/cookbook/graph_transformations.rs:selective_logging}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__graph_transformations__tests__selective_logging.snap:5:}}
```

## 4. Caching and diamond dependencies

Diamond dependency graph: `compile` and `link` both depend on `stdlib`.
Without caching, `stdlib` is visited twice:

```rust
{{#include ../../../src/cookbook/graph_transformations.rs:caching}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__graph_transformations__tests__caching.snap:5:}}
```

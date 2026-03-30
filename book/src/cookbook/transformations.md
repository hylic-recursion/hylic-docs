# Transformations

All examples share one domain, one graph, one base fold.
Each transformation wraps an existing piece — the base is never modified.

## Domain

```rust
{{#include ../../../src/cookbook/transformations.rs:domain}}
```

## The base fold

Sum all task costs bottom-up. This is the starting point for every
transformation below:

```rust
{{#include ../../../src/cookbook/transformations.rs:base}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__transformations__tests__base.snap:5:}}
```

## map_init: wrap init to add logging

The base fold is untouched — `logged_sum` is a new fold that logs
each node as it's visited, then delegates to the original init:

```rust
{{#include ../../../src/cookbook/transformations.rs:map_init}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__transformations__tests__map_init.snap:5:}}
```

## map_finalize: post-process each result

Cap each subtree's total at 500ms. Children accumulate normally —
the cap applies after finalize, so parents see the clamped value:

```rust
{{#include ../../../src/cookbook/transformations.rs:map_finalize}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__transformations__tests__map_finalize.snap:5:}}
```

## zipmap: per-node annotation

Derive a classification from each subtree's sum. The accumulation
is still sum — zipmap post-processes per node, producing (R, Extra):

```rust
{{#include ../../../src/cookbook/transformations.rs:zipmap}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__transformations__tests__zipmap.snap:5:}}
```

## map: change the result type

Transform u64 → String. The backmapper lets children's String results
flow back through the original u64 accumulator:

```rust
{{#include ../../../src/cookbook/transformations.rs:map_result}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__transformations__tests__map_result.snap:5:}}
```

## Graph: memoize diamond dependencies

Same fold, wrap the graph. `memoize_treeish_by` caches children by
a key function — on repeat visits, the graph function is skipped:

```rust
{{#include ../../../src/cookbook/transformations.rs:memoize}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__transformations__tests__memoize.snap:5:}}
```

## Composition: stack transforms

Transforms compose by chaining. Each wraps the previous —
no rewriting, no touching the base:

```rust
{{#include ../../../src/cookbook/transformations.rs:composed}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__transformations__tests__composed.snap:5:}}
```

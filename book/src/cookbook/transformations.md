# Transformations

Features as standalone functions that match the transformation contract.
One domain, one base fold, one base graph. Each feature is defined
separately, then plugged in with a single method call.

## Domain and base

```rust
{{#include ../../../src/cookbook/transformations.rs:domain}}
```

## Fold phase wrappers

### map_init: visit_logger

A function returning an init wrapper. It IS the logging feature —
defined once, plugged into any fold:

```rust
{{#include ../../../src/cookbook/transformations.rs:visit_logger}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__transformations__tests__visit_logger.snap:5:}}
```

### map_accumulate: skip_small_children

An accumulate wrapper that filters during folding — children below
a threshold are not accumulated:

```rust
{{#include ../../../src/cookbook/transformations.rs:skip_small}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__transformations__tests__skip_small.snap:5:}}
```

### map_finalize: clamp_at

A finalize wrapper that caps each subtree's result. Parents see
the clamped value:

```rust
{{#include ../../../src/cookbook/transformations.rs:clamp_at}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__transformations__tests__clamp_at.snap:5:}}
```

## Fold result augmentation

### zipmap: classify

A plain function matching the zipmap contract (`Fn(&R) -> RZip`).
Each subtree's result is paired with a category:

```rust
{{#include ../../../src/cookbook/transformations.rs:classify}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__transformations__tests__classify.snap:5:}}
```

## Graph transformations

### Edge filtering: only_costly_deps

A graph transformation that prunes edges. Takes a Treeish, returns
a Treeish — same node type, fewer children:

```rust
{{#include ../../../src/cookbook/transformations.rs:only_costly}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__transformations__tests__only_costly.snap:5:}}
```

### Memoization: cache diamond dependencies

Same fold, wrapped graph. On repeat visits, cached children are
returned without calling the graph function:

```rust
{{#include ../../../src/cookbook/transformations.rs:memoize}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__transformations__tests__memoize.snap:5:}}
```

## Composition

Three independent features chained on one base fold. Each wraps
the previous — no rewriting, concerns stay separated:

```rust
{{#include ../../../src/cookbook/transformations.rs:composed}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__transformations__tests__composed.snap:5:}}
```

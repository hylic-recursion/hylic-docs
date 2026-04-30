# Quick Start

A complete fold — definition, tree structure, sequential execution
— is one prelude line and three closures:

```rust
{{#include ../../src/docs_examples.rs:intro_dir_example}}
```

`fold(...)` builds a Shared-domain `Fold<Dir, u64, u64>` from three
closures: `init` produces a per-node heap from a `&Dir`,
`accumulate` folds each child's result into the heap, and
`finalize` extracts the result. `treeish(...)` wraps a children
function as a `Treeish<Dir>`. `FUSED` is the sequential executor
constant — callback-based recursion, no overhead beyond the fold
closures.

The Funnel executor swaps in without touching the fold or graph:

```rust
{{#include ../../src/docs_examples.rs:quickstart_funnel}}
```

`Spec::default(n)` picks the Robust preset over `n` worker threads;
see [Funnel policies](./funnel/policies.md) for the alternatives.

For repeated folds, pool creation amortises in a session scope:

```rust
{{#include ../../src/docs_examples.rs:quickstart_session}}
```

## The same fold over flat data

The tree need not live inside the data. The same summation fold
runs over a `Vec<Vec<usize>>` adjacency list, where nodes are
integer indices:

```rust
{{#include ../../src/docs_examples.rs:intro_flat_example}}
```

Only the node type and the Treeish change — the fold logic is
identical. This separation is the foundation of hylic's
composability.

## Pivoting between the two

The two formulations describe the same shape of computation in
different node types. `Fold::contramap_n` lets you take a fold
written for one and run it over the other, without rewriting any
of the fold's closures.

Going Dir-fold → flat: synthesise a minimal `Dir` per index — only
the fields the fold actually reads need to exist. The graph is
swapped on the executor's side.

```rust
{{#include ../src/docs_examples.rs:pivot_dir_fold_to_flat}}
```

The mirror direction projects each `Dir` to the index the
flat-fold expects:

```rust
{{#include ../src/docs_examples.rs:pivot_flat_fold_to_dir}}
```

In both directions the original fold's closures pass through
unchanged; the only transformation is `contramap_n` on the input
axis. The graph is chosen at the call site to match.

## Further reading

- [The recursive pattern](./concepts/separation.md) — the
  decomposition that makes this work
- [Fold guide](./guides/fold.md) — transformations: map, contramap,
  product, phase wrapping
- [Graph guide](./guides/treeish.md) — filtering, contramap, memoization
- [Funnel executor](./funnel/overview.md) — the parallel work-stealing engine
- [Cookbook](./cookbook/fibonacci.md) — worked examples

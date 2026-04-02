# Graph: controlling traversal

The graph — `Treeish<N>` or `Edgy<N, E>` — determines which children
each node has. Transform the graph to change what gets visited,
without touching the fold.

## Constructors

Three ways to create a `Treeish<N>`:

```rust
{{#include ../../../src/docs_examples.rs:treeish_constructors}}
```

Prefer `treeish_visit` for performance — no Vec allocation per node.

## Edge transformations

<!-- -->

```dot process
digraph {
    rankdir=LR;
    node [shape=box, style="rounded,filled", fillcolor="#f5f5f5", fontname="monospace", fontsize=11];
    edge [fontname="sans-serif", fontsize=10];

    E1 [label="Edgy<N, E>"];
    E2 [label="Edgy<N, E2>\nmap(f)"];
    E3 [label="Edgy<N2, E>\ncontramap(f)"];
    E4 [label="Edgy<N, E>\nfilter(pred)"];

    E1 -> E2 [label="transform edges"];
    E1 -> E3 [label="change node type"];
    E1 -> E4 [label="prune edges"];
}
```

### filter — prune children

<!-- -->

```rust
{{#include ../../../src/docs_examples.rs:graph_filter}}
```

Same fold, fewer children. The fold doesn't know about the pruning.

## Caching: memoize_treeish

For DAGs (directed acyclic graphs) where the same node appears
multiple times, `memoize_treeish` caches the children computation:

```rust
{{#include ../../../src/docs_examples.rs:memoize_example}}
```

The first visit to a node computes and caches its children. Subsequent
visits return the cached result.

## Visit combinator

`Edgy::at(node)` returns a `Visit<T, F>` — a zero-allocation
push-based iterator. Supports `map`, `filter`, `fold`, `count`,
`collect_vec`. All callback-based internally — no intermediate
allocations.

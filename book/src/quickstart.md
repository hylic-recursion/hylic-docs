# Quick Start

This page walks through constructing and running a fold over a
simple tree, then switching to parallel execution.

## Define the tree type

Any Rust type whose nodes have children works. hylic does not
require a particular trait — only that you can describe the
parent-child relationship as a function.

```rust
use hylic::domain::shared as dom;
use hylic::graph;

#[derive(Clone)]
struct Dir {
    name: String,
    size: u64,
    children: Vec<Dir>,
}
```

## Describe the tree structure

A `Treeish<Dir>` encapsulates how to traverse from a node to its
children. The `treeish` constructor wraps a function that returns
a `Vec` of children; `treeish_visit` accepts a callback-based form
that avoids the intermediate allocation.

```rust
let graph = graph::treeish(|d: &Dir| d.children.clone());
```

## Define the fold

A fold has three phases: `init` produces a heap value from a node,
`accumulate` incorporates each child's result into the heap, and
`finalize` extracts the result from the heap. `simple_fold` is a
shorthand for the common case where the heap type equals the result
type and finalize is the identity.

```rust
let init = |d: &Dir| d.size;
let acc = |heap: &mut u64, child: &u64| *heap += child;
let fold = dom::simple_fold(init, acc);
```

## Run it

`dom::FUSED` is the sequential executor. It recurses through the
tree using the fold and graph provided, with no additional
allocation or indirection beyond what the fold closures themselves
require.

```rust
let tree = Dir {
    name: "root".into(), size: 10,
    children: vec![
        Dir { name: "src".into(), size: 100, children: vec![
            Dir { name: "main.rs".into(), size: 50, children: vec![] },
        ]},
        Dir { name: "docs".into(), size: 30, children: vec![] },
    ],
};

let total = dom::FUSED.run(&fold, &graph, &tree);
assert_eq!(total, 190); // 10 + 100 + 50 + 30
```

## Switch to parallel execution

The Funnel executor uses the same fold and graph. It creates a
scoped thread pool internally, distributes subtrees across workers
via CPS work-stealing, and joins before returning.

```rust
use hylic::cata::exec::funnel;

let total = dom::exec(funnel::Spec::default(8)).run(&fold, &graph, &tree);
assert_eq!(total, 190);
```

For repeated folds, pool creation can be amortized by entering a
session scope:

```rust
dom::exec(funnel::Spec::default(8)).session(|s| {
    let total1 = s.run(&fold, &graph, &tree);
    let total2 = s.run(&fold, &graph, &tree);
    // The thread pool is shared across both folds.
});
```

## Further reading

- [The recursive pattern](./concepts/separation.md) — the
  decomposition that makes this work
- [Fold guide](./guides/fold.md) — transformations: map, contramap,
  product, phase wrapping
- [Graph guide](./guides/graph.md) — filtering, contramap, memoization
- [Funnel executor](./funnel/overview.md) — the parallel CPS engine
- [Cookbook](./cookbook/fibonacci.md) — worked examples

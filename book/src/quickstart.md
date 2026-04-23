# Quick Start

This page walks through constructing and running a fold, first over
a nested struct, then over a flat adjacency list — same fold, same
result, different data representation.

## Define a node type

Any Rust type can serve as a node. The tree structure is defined
externally by a `Treeish` function, not by the data itself. For this
first example, the node type is a struct with a children field:

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

A `Treeish<Dir>` is a function from a node to its children. The
`treeish` constructor wraps a `Vec`-returning function; `treeish_visit`
accepts a callback form that avoids the intermediate allocation.

```rust
let graph = graph::treeish(|d: &Dir| d.children.clone());
```

## Define the fold

A fold has three phases: `init` produces a heap value from a node,
`accumulate` incorporates each child's result into the heap, and
`finalize` extracts the result. `simple_fold` is a shorthand for
the common case where the heap type equals the result type and
finalize is the identity.

```rust
let init = |d: &Dir| d.size;
let acc = |heap: &mut u64, child: &u64| *heap += child;
let fold = dom::simple_fold(init, acc);
```

## Run it

`dom::FUSED` is the sequential executor — callback-based recursion,
no overhead beyond the fold closures.

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

## The same fold over flat data

The tree need not be nested. Here the same summation fold runs over
a `Vec<Vec<usize>>` adjacency list, where nodes are integer indices:

```rust
let children = vec![vec![1, 2], vec![3], vec![], vec![]];
let sizes = vec![10u64, 100, 50, 30];

let graph = graph::treeish_visit(move |n: &usize, cb: &mut dyn FnMut(&usize)| {
    for &c in &children[*n] { cb(&c); }
});
let fold = dom::simple_fold(
    move |n: &usize| sizes[*n],
    |heap: &mut u64, child: &u64| *heap += child,
);

let total = dom::FUSED.run(&fold, &graph, &0);
assert_eq!(total, 190); // same tree, same result
```

The fold logic is identical — only the node type and the treeish
change. This separation is the foundation of hylic's composability.

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
- [Graph guide](./guides/treeish.md) — filtering, contramap, memoization
- [Funnel executor](./funnel/overview.md) — the parallel work-stealing engine
- [Cookbook](./cookbook/fibonacci.md) — worked examples

# Quick Start

Define a tree type, write a fold, run it. Five minutes.

## 1. Define your tree

```rust
use hylic::domain::shared as dom;

#[derive(Clone)]
struct Dir {
    name: String,
    size: u64,
    children: Vec<Dir>,
}
```

## 2. Tell hylic how to find children

```rust
let graph = dom::treeish(|d: &Dir| d.children.clone());
```

This creates a `Treeish<Dir>` — a callback-based traversal function.
No allocation per visit when using `treeish_visit` (the callback
form); `treeish` wraps a Vec-returning function for convenience.

## 3. Define the fold

A fold has three phases: init (node → heap), accumulate (heap × child
result → heap), finalize (heap → result).

```rust
let init = |d: &Dir| d.size;
let acc = |heap: &mut u64, child: &u64| *heap += child;
let fold = dom::simple_fold(init, acc);
```

`simple_fold` is shorthand for when the heap type equals the result
type and finalize is identity.

## 4. Run it

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

`dom::FUSED` is the sequential executor — zero overhead, callback-based
recursion. Same fold, same graph, same result as hand-written recursion.

## 5. Make it parallel

```rust
use hylic::cata::exec::funnel;

let total = dom::exec(funnel::Spec::default(8)).run(&fold, &graph, &tree);
assert_eq!(total, 190); // same result, concurrent execution
```

Same `.run()` call. The Funnel executor creates a thread pool
internally, processes subtrees concurrently via CPS work-stealing,
and joins. The fold and graph are unchanged.

For repeated folds, amortize pool creation:

```rust
dom::exec(funnel::Spec::default(8)).session(|s| {
    let total1 = s.run(&fold, &graph, &tree);
    let total2 = s.run(&fold, &graph, &tree);
    // pool shared across both folds
});
```

## What next

- [The recursive pattern](./concepts/separation.md) — why the
  three-phase separation matters
- [Fold: shaping the computation](./guides/fold.md) — map, zipmap,
  contramap, product
- [Graph: controlling traversal](./guides/graph.md) — filter,
  contramap, memoize
- [Funnel executor](./funnel/overview.md) — CPS walk, policies,
  benchmarks
- [Cookbook](./cookbook/fibonacci.md) — working examples with
  snapshot-tested output

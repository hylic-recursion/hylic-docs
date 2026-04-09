# hylic

Composable recursive tree computation for Rust.

You write `init`, `accumulate`, `finalize` once. You can then run the
fold sequentially, in parallel, with tracing, on filtered trees, on
lazily-discovered dependency graphs — without changing a line of fold
logic.

```rust
use hylic::domain::shared as dom;

#[derive(Clone)]
struct Dir { name: String, size: u64, children: Vec<Dir> }

let graph = dom::treeish(|d: &Dir| d.children.clone());
let fold = dom::simple_fold(
    |d: &Dir| d.size,
    |heap: &mut u64, child: &u64| *heap += child,
);

let tree = Dir {
    name: "project".into(), size: 10,
    children: vec![
        Dir { name: "src".into(), size: 200, children: vec![] },
        Dir { name: "docs".into(), size: 50, children: vec![] },
    ],
};

// Sequential:
let total = dom::FUSED.run(&fold, &graph, &tree);
assert_eq!(total, 260);

// Parallel (same fold, same graph):
use hylic::cata::exec::funnel;
let total = dom::exec(funnel::Spec::default(4)).run(&fold, &graph, &tree);
assert_eq!(total, 260);
```

The fold defines **what to compute** (sum sizes). The graph defines
**the tree structure** (children of each Dir). The executor defines
**how to traverse** (sequential or parallel). Each is independent —
change one without touching the others.

## Key ideas

- **Three-phase fold**: `init` / `accumulate` / `finalize` through an
  intermediate heap type `H`. Composable via
  [transformations](./concepts/transforms.md): map, zipmap, contramap,
  product.
- **Push-based traversal**: `graph.visit(&node, |child| ...)` — zero
  allocation per node. No `Vec<Child>` collected unless you ask.
- **Uniform execution**: every executor has `.run()`. Sequential,
  parallel, with-tracing — same call, same interface. See
  [The Exec pattern](./executor-design/exec_pattern.md).
- **Boxing domains**: three storage strategies
  ([Shared, Local, Owned](./design/domains.md)) control how closures
  are stored. The domain lives on the executor, not the fold.
- **Funnel**: a CPS work-stealing parallel executor with compile-time
  policy selection. See [Funnel](./funnel/overview.md).

## Start here

→ [Quick Start](./quickstart.md) — your first fold in 5 minutes

Then: [The recursive pattern](./concepts/separation.md) to understand
the core design, or the [Cookbook](./cookbook/fibonacci.md) for working
examples.

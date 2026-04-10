# hylic

A Rust library for composable recursive tree computation.

hylic separates a recursive computation into three independent
concerns: a *fold* that defines what to compute at each node, a
*graph* that describes the tree structure, and an *executor* that
controls how the recursion is carried out. Each concern can be
defined, transformed, and composed independently of the others.

```rust
use hylic::domain::shared as dom;
use hylic::graph;

#[derive(Clone)]
struct Dir { name: String, size: u64, children: Vec<Dir> }

let graph = graph::treeish(|d: &Dir| d.children.clone());
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

// Sequential execution:
let total = dom::FUSED.run(&fold, &graph, &tree);
assert_eq!(total, 260);

// Parallel execution — the fold and graph are unchanged:
use hylic::cata::exec::funnel;
let total = dom::exec(funnel::Spec::default(4)).run(&fold, &graph, &tree);
assert_eq!(total, 260);
```

The fold defines what to compute (sum the sizes). The graph defines
the tree structure (children of each `Dir`). The executor determines
the traversal strategy (sequential or parallel). Replacing one leaves
the others untouched.

## Core concepts

**Three-phase fold.** A fold consists of `init`, `accumulate`, and
`finalize`, mediated by a heap type `H`. Folds are composable through
[transformations](./concepts/transforms.md): map, contramap, product,
and phase-wrapping combinators.

**Push-based traversal.** The graph exposes children through a
callback: `graph.visit(&node, |child| ...)`. This avoids allocating
a `Vec` of children per node. Graph types live in `hylic::graph` and
are domain-independent.

**Uniform execution.** Every executor — sequential, parallel, or
user-defined — presents the same `.run()` interface. Resource
management (thread pools, arenas) is an internal concern of the
executor. See [The Exec pattern](./executor-design/exec_pattern.md).

**Boxing domains.** Three storage strategies control how fold
closures are boxed: Shared (Arc), Local (Rc), and Owned (Box). The
domain is a type parameter on the executor, not on the fold or graph.
See [Domain system](./design/domains.md).

**Funnel executor.** A parallel CPS work-stealing executor with
compile-time policy selection across three behavioral axes. See
[Funnel](./funnel/overview.md).

## Where to start

The [Quick Start](./quickstart.md) walks through constructing and
running a fold. [The recursive pattern](./concepts/separation.md)
explains the underlying decomposition. The [Cookbook](./cookbook/fibonacci.md)
contains worked examples for common patterns.

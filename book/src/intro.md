# hylic

Composable recursive tree computation for Rust.

hylic separates **what to compute** (Fold) from **the tree
structure** (Treeish) and **how to execute** (Executor). Each piece
is independently definable, transformable, and composable.

```rust
use hylic::domain::shared as dom;

// Define the computation: three closures
let init = |n: &i32| *n as u64;
let acc  = |h: &mut u64, c: &u64| *h += c;
let fold = dom::simple_fold(init, acc);

// Define the tree structure
let graph = dom::treeish(|n: &i32| if *n > 1 { vec![n - 1, n - 2] } else { vec![] });

// Execute: fold + graph + root → result
let result = dom::FUSED.run(&fold, &graph, &5);
```

Three [boxing domains](./design/domains.md) (Shared, Local, Owned)
control how closures are stored — from parallel-ready Arc to
zero-overhead Box. The domain lives on the executor, not the data
types.

Start with [The recursive pattern](./concepts/separation.md)
to understand the core idea, then explore the
[Cookbook](./cookbook/fibonacci.md) for working examples.

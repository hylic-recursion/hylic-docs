# hylic

Composable recursive tree computation for Rust.

hylic separates **what to compute** (Fold) from **the tree
structure** (Treeish) and **how to execute** (Executor). Each piece
is independently definable, transformable, and composable.

Three [boxing domains](./design/domains.md) (Shared, Local, Owned)
control how closures are stored — from parallel-ready Arc to
zero-overhead Box. The domain lives on the executor, not the data
types.

Start with [The recursive pattern](./concepts/separation.md)
to understand the core idea, then explore the
[Cookbook](./cookbook/fibonacci.md) for working examples.

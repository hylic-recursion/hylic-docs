# hylic

Composable recursive tree computation for Rust.

hylic separates **what to compute** (Fold) from **the tree
structure** (Treeish) and **how to execute** (Exec). Each piece
is independently definable, transformable, and composable.

Start with [The recursive pattern](./concepts/separation.md)
to understand the core idea, then explore the
[Cookbook](./cookbook/fibonacci.md) for working examples.

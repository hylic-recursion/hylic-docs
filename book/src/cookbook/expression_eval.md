# Expression evaluation

Evaluate an AST bottom-up. Uses `vec_fold` to inspect all child
results during finalize.

> **Imports:** `hylic::prelude::vec_fold` + `VecHeap`, `hylic::graph::treeish_visit`, `hylic::cata::Strategy`

```rust
{{#include ../../../src/cookbook/expression_eval.rs}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__expression_eval__tests__expr_eval.snap:5:}}
```

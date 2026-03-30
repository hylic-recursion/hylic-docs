# Expression evaluation

Fold an arithmetic expression tree (AST) bottom-up. Each node type
(Num, Add, Mul, Neg) handles its children differently. `vec_fold`
gives the finalize function access to the node and all child results.

```rust
{{#include ../../../src/cookbook/expression_eval.rs:expression_eval}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__expression_eval__tests__expression_eval_result.snap:5:}}
```

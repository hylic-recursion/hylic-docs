# Expression evaluation

Evaluate an AST bottom-up. `vec_fold` gives finalize access to
the node and all child results — needed when different node types
combine children differently.

```rust
{{#include ../../../src/cookbook/expression_eval.rs}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__expression_eval__tests__evaluate_expression.snap:5:}}
```

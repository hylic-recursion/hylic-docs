# Fibonacci

The simplest hylic example. Fibonacci numbers form a degenerate tree
(each node branches into n-1 and n-2). The fold sums leaf values bottom-up.

Intentionally naive — demonstrates the mechanics, not performance.

```rust
{{#include ../../../src/cookbook/fibonacci.rs:fibonacci}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__fibonacci__tests__fibonacci_result.snap:5:}}
```

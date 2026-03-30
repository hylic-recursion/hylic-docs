# Configuration inheritance

Overlay configuration scopes bottom-up — child overrides accumulate,
but the parent's own values always win.

> **Imports:** `hylic::fold::simple_fold`, `hylic::graph::treeish_from`, `hylic::cata::Strategy`

```rust
{{#include ../../../src/cookbook/config_inheritance.rs}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__config_inheritance__tests__config.snap:5:}}
```

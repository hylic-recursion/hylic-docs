# Configuration inheritance

Overlay configuration scopes bottom-up. Each scope has its own overrides
and child scopes. The fold merges children's keys upward, but the parent's
own values always win.

```rust
{{#include ../../../src/cookbook/config_inheritance.rs:config_inheritance}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__config_inheritance__tests__config.snap:5:}}
```

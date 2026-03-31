# Configuration inheritance

Overlay configuration scopes bottom-up. `or_insert` in accumulate
gives parent-wins semantics — init runs before accumulate, so the
parent's values are already in the map.

```rust
{{#include ../../../src/cookbook/config_inheritance.rs}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__config_inheritance__tests__config.snap:5:}}
```

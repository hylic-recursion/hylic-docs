# Module resolution

The seed-based graph pattern. `SeedGraph` is a general anamorphism —
three functions define how to unfold a tree from seeds. For the
fallible case (where growing can fail), `seeds_for_fallible` lifts
a valid-only seed function to handle `Either<Error, Valid>` nodes.

```rust
{{#include ../../../src/cookbook/module_resolution.rs}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__module_resolution__tests__resolve_modules.snap:5:}}
```

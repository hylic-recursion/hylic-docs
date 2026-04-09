# Transformations

Features as standalone functions matching the transformation contract.
One domain, one base fold, one base graph. Each feature is a named
function — defined separately, plugged in with a single method call.

The phase-wrapping contract — each wrapper receives the original
phase as a callable reference:
- `wrap_init`: `Fn(&N, &dyn Fn(&N) -> H) -> H`
- `wrap_accumulate`: `Fn(&mut H, &R, &dyn Fn(&mut H, &R))`
- `wrap_finalize`: `Fn(&H, &dyn Fn(&H) -> R) -> R`

```rust
{{#include ../../../src/cookbook/transformations.rs}}
```

Outputs:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__transformations__tests__visit_logger.snap:5:}}
```
```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__transformations__tests__skip_small.snap:5:}}
```
```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__transformations__tests__clamp_at.snap:5:}}
```
```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__transformations__tests__classify.snap:5:}}
```
```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__transformations__tests__only_costly.snap:5:}}
```
```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__transformations__tests__memoize.snap:5:}}
```
```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__transformations__tests__composed.snap:5:}}
```

# Transformations

Features as standalone functions matching the transformation contract.
One domain, one base fold, one base graph. Each feature is a named
function — defined separately, plugged in with a single method call.

The fold type aliases used in the contract signatures:
- `InitFn<N, H>` = `Box<dyn Fn(&N) -> H + Send + Sync>`
- `AccumulateFn<H, R>` = `Box<dyn Fn(&mut H, &R) + Send + Sync>`
- `FinalizeFn<H, R>` = `Box<dyn Fn(&H) -> R + Send + Sync>`

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

# Transformations

Features as standalone functions matching the transformation contract.
One domain, one base fold, one base graph. Each feature is defined
separately, then plugged in with a single method call.

> **Fold type aliases** (from `hylic::fold`):
> - `InitFn<N, H>` = `Box<dyn Fn(&N) -> H + Send + Sync>`
> - `AccumulateFn<H, R>` = `Box<dyn Fn(&mut H, &R) + Send + Sync>`
> - `FinalizeFn<H, R>` = `Box<dyn Fn(&H) -> R + Send + Sync>`
>
> **Fold transforms** (methods on `hylic::fold::Fold`):
> `map_init`, `map_accumulate`, `map_finalize`, `zipmap`, `contramap`, `product`
>
> **Graph transforms**: `Edgy::filter` (from `hylic::graph`),
> `memoize_treeish_by` (from `hylic::prelude`)

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

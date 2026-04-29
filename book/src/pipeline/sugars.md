# Sugars — the chainable surface

Every transform users reach for at Stage 1 or Stage 2 is a
trait method. Each method picks an axis, builds the right
[library lift](../concepts/lifts.md), and either reshapes the
Stage-1 slots in place (Stage 1) or appends the lift to the
chain via `then_lift` (Stage 2).

## Trait surfaces

| Where you are                                | Sugars in scope                      |
|----------------------------------------------|--------------------------------------|
| `SeedPipeline<Shared, …>`                    | [`SeedSugarsShared`](../../../../hylic-pipeline/src/sugars/seed_shared.rs) |
| `SeedPipeline<Local, …>`                     | [`SeedSugarsLocal`](../../../../hylic-pipeline/src/sugars/seed_local.rs)   |
| `TreeishPipeline<Shared, …>`                 | [`TreeishSugarsShared`](../../../../hylic-pipeline/src/sugars/treeish_shared.rs) |
| `TreeishPipeline<Local, …>`                  | [`TreeishSugarsLocal`](../../../../hylic-pipeline/src/sugars/treeish_local.rs)   |
| `Stage2Pipeline<Base, L>` (Shared, any Base) | [`Stage2SugarsShared`](../../../../hylic-pipeline/src/sugars/stage2_shared.rs) |
| `Stage2Pipeline<Base, L>` (Local, any Base)  | [`Stage2SugarsLocal`](../../../../hylic-pipeline/src/sugars/stage2_local.rs)   |

`use hylic_pipeline::prelude::*;` brings them all in scope.

## Shared and Local: same names, different bounds

Method names are identical across domains. Only the closure
storage and bounds differ:

```text
// Shared: parallel-safe; closures must be Send + Sync.
let r = shared_pipe.wrap_init(w).zipmap(m).run(...);

// Local: same call shape, captures may be non-Send.
let r = local_pipe.wrap_init(w).zipmap(m).run(...);
```

## Stage 2: one trait covers both Bases

`Stage2SugarsShared` is one trait, blanket-implemented on every
`Stage2Pipeline<Base, L>`. The treeish-rooted vs seed-rooted
dispatch happens inside the lift-construction call, not at the
trait level. Every Stage-2 sugar body is one line:

```rust
{{#include ../../../../hylic-pipeline/src/sugars/stage2_shared.rs:stage2_sugars_wrap_init}}
```

`<<Self::Base as Stage2Base>::Wrap as WrapShared>::build_wrap_init`
is the dispatch. `Identity` (treeish-rooted) calls
`Shared::wrap_init_lift` directly; `SeedWrap` (seed-rooted)
wraps the user's closure with a `SeedNode::Node(_)`-peeling
adapter, then calls the same `Shared::wrap_init_lift`. Both
produce a `ShapeLift`; both forward to `then_lift`. From the
user's perspective the closure types at `&UN` either way. See
[Wrap dispatch](./wrap_dispatch.md) for the full mechanics.

## Stage 1: per-Base reshape sugars

Stage-1 reshape rewrites the base slots in place and returns a
fresh Stage-1 pipeline of (possibly different) type parameters:

```rust
{{#include ../../../../hylic-pipeline/src/sugars/seed_shared.rs:seed_sugars_shared_trait}}
```

Stage-2 sugars are not in scope until `.lift()` (or the
TreeishPipeline auto-lift) has produced a `Stage2Pipeline`.

## Catalogue

### Stage 1 — `SeedSugarsShared` / `SeedSugarsLocal`

Operates on `SeedPipeline<D, N, Seed, H, R>`:

| method                    | output shape                       |
|---------------------------|------------------------------------|
| `filter_seeds(pred)`      | `SeedPipeline<D, N, Seed, H, R>`   |
| `wrap_grow(w)`            | `SeedPipeline<D, N, Seed, H, R>`   |
| `map_node_bi(co, contra)` | `SeedPipeline<D, N2, Seed, H, R>`  |
| `map_seed_bi(to, from)`   | `SeedPipeline<D, N, Seed2, H, R>`  |

### Stage 1 — `TreeishSugarsShared` / `TreeishSugarsLocal`

Operates on `TreeishPipeline<D, N, H, R>`:

| method                    | output shape                       |
|---------------------------|------------------------------------|
| `map_node_bi(co, contra)` | `TreeishPipeline<D, N2, H, R>`     |

### Stage 2 — `Stage2SugarsShared` / `Stage2SugarsLocal`

Operates on `Stage2Pipeline<Base, L>` (and on `TreeishPipeline`
via auto-lift). User closures type at `&UN`; the chain's actual
N is `UN` (treeish-rooted) or `SeedNode<UN>` (seed-rooted),
bridged by Wrap.

| method                            | what the lift does                                  |
|-----------------------------------|-----------------------------------------------------|
| `wrap_init(w)`                    | intercept `init` at every node                      |
| `wrap_accumulate(w)`              | intercept `accumulate`                              |
| `wrap_finalize(w)`                | intercept `finalize`                                |
| `zipmap(m)`                       | extend `R`: `R → (R, Extra)`                        |
| `map_r_bi(fwd, bwd)`              | change `R` bijectively                              |
| `filter_edges(pred)`              | drop edges from the graph                           |
| `wrap_visit(w)`                   | intercept graph `visit`                             |
| `memoize_by(key)`                 | memoise subtree results by key                      |
| `map_n_bi(co, contra)`            | change `N` bijectively (chain-tip)                  |
| `explain()`                       | wrap fold with per-node trace recording             |
| `explain_describe(fmt, emit)`     | streaming trace; chain-tip `R` unchanged (Shared only) |

The Stage-1 reshape `map_node_bi` and the Stage-2 sugar
`map_n_bi` share a purpose (change `N`) but are distinct
operations. Stage 1 rewrites the base slots in place; Stage 2
composes a `ShapeLift` onto the chain. Use Stage 2 when the N
change must sit on top of earlier sugars.

## Where `wrap_init`'s second argument comes from

Every `wrap_*` user closure receives an `orig: &dyn Fn(...) -> ...`
parameter alongside the node. `orig` is the prior fold's
corresponding phase, exposed as a value so the sugar body can
compose with it: `|n, orig| orig(n) + 1`. Lifts are, at the
type level, natural transformations between fold algebras; a
phase mapper takes the prior phase as input and produces the
new phase. See
[the type-level deep dive](../design/type_level.md#wrap_init-as-a-phase-mapper).

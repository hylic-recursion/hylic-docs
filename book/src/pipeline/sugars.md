# Sugars — the chainable surface

Every transform a user reaches for at Stage 1 or Stage 2 is exposed via a
trait method. Each method is one line: it picks an axis, builds the right
[library lift](../concepts/lifts.md), and forwards through the pipeline's
sole compositional primitive (`then_lift` for Stage 2; the per-base
reshape ctor for Stage 1).

## The four sugar surfaces

Six trait files: two stages × two domains, plus Stage-1 split per Base.

| Where you are                                | Sugars in scope                      |
|----------------------------------------------|--------------------------------------|
| `SeedPipeline<Shared, …>`                    | [`SeedSugarsShared`](../../../../hylic-pipeline/src/sugars/seed_shared.rs) |
| `SeedPipeline<Local, …>`                     | [`SeedSugarsLocal`](../../../../hylic-pipeline/src/sugars/seed_local.rs)   |
| `TreeishPipeline<Shared, …>`                 | [`TreeishSugarsShared`](../../../../hylic-pipeline/src/sugars/treeish_shared.rs) |
| `TreeishPipeline<Local, …>`                  | [`TreeishSugarsLocal`](../../../../hylic-pipeline/src/sugars/treeish_local.rs)   |
| `Stage2Pipeline<Base, L>` (Shared, any Base) | [`Stage2SugarsShared`](../../../../hylic-pipeline/src/sugars/stage2_shared.rs) |
| `Stage2Pipeline<Base, L>` (Local, any Base)  | [`Stage2SugarsLocal`](../../../../hylic-pipeline/src/sugars/stage2_local.rs)   |

`use hylic_pipeline::prelude::*;` brings them all in scope.

## Shared/Local: same names, different bounds

Domain mirrors are the only place sugars duplicate. The trait bodies
read identically; only the closure-storage cell type (`Arc` vs `Rc`) and
closure-bound (`Send + Sync` vs none) differ:

```text
// Shared: parallel-safe; closures must be Send + Sync.
let r = shared_pipe.wrap_init(w).zipmap(m).run(...);

// Local: same call shape, captures may be non-Send (e.g. Rc<RefCell<…>>).
let r = local_pipe.wrap_init(w).zipmap(m).run(...);
```

The duplication is one of three [accepted-debt
items](../../../hylic/KB/.plans/finishing-up/post-split-review/ACCEPTED-DEBT.md)
in the codebase. Collapsing it cleanly would require macros, which the
codebase declines.

## Stage 2: one trait covers both Bases

`Stage2SugarsShared` is **one** trait, blanket-implemented on every
`Stage2Pipeline<Base, L>`. The chain dispatch (treeish-rooted vs
seed-rooted) happens inside the lift-construction call, not at the trait
level. Concretely, every Stage-2 sugar body is one line:

```rust
{{#include ../../../../hylic-pipeline/src/sugars/stage2_shared.rs:stage2_sugars_wrap_init}}
```

`<<Self::Base as Stage2Base>::Wrap as WrapShared>::build_wrap_init` is
where the dispatch lives. `Identity` (treeish-rooted) calls
`Shared::wrap_init_lift` directly; `SeedWrap` (seed-rooted) wraps the
user's closure with a `SeedNode::Node(_)`-peeling adapter, then calls the
same `Shared::wrap_init_lift` on the wrapped chain. From the user's
perspective both forms accept `&UN` user closures and behave identically.
The seed-rooted chain just routes `EntryRoot` through silently. See
[Wrap dispatch](./wrap_dispatch.md) for the full mechanics.

## Stage 1: per-Base reshape sugars

Stage-1 reshape doesn't go through a chain — it rewrites the Stage-1
slots in place and returns a new Stage-1 pipeline. Two trait files per
Base, one per domain:

```rust
{{#include ../../../../hylic-pipeline/src/sugars/seed_shared.rs:seed_sugars_shared_trait}}
```

Each method returns a fresh `SeedPipeline` (or `TreeishPipeline`) of
possibly different type parameters. Stage-1 transforms preserve the
"base slots" form; Stage-2 sugars are not yet available because
`.lift()` hasn't been called.

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

(`TreeishPipeline` has fewer reshape sugars: no seeds to filter, no grow
to wrap.)

### Stage 2 — `Stage2SugarsShared` / `Stage2SugarsLocal`

Operates on `Stage2Pipeline<Base, L>` (and on `TreeishPipeline<…>` via
auto-lift). User closures type at `&UN`; the chain's actual N is
`UN` (treeish-rooted) or `SeedNode<UN>` (seed-rooted), bridged by Wrap.

| method                            | what the lift does                                  |
|-----------------------------------|-----------------------------------------------------|
| `wrap_init(w)`                    | intercept `init` at every node                      |
| `wrap_accumulate(w)`              | intercept `accumulate`                              |
| `wrap_finalize(w)`                | intercept `finalize`                                |
| `zipmap(m)`                       | extend R: `R → (R, Extra)`                          |
| `map_r_bi(fwd, bwd)`              | change R bijectively                                |
| `filter_edges(pred)`              | drop edges from the graph                           |
| `wrap_visit(w)`                   | intercept graph `visit`                             |
| `memoize_by(key)`                 | memoise subtree results by key                      |
| `map_n_bi(co, contra)`            | change N bijectively (chain-tip)                    |
| `explain()`                       | wrap fold with per-node trace recording             |
| `explain_describe(fmt, emit)`     | streaming trace; chain-tip R unchanged (Shared only) |

On a seed-rooted chain, every `&UN` closure runs only against real
nodes; the `EntryRoot` row is auto-routed to the chain's `orig`
continuation by the [Wrap machinery](./wrap_dispatch.md). The user
never mentions `EntryRoot` or pattern-matches `SeedNode`.

The Stage-1 reshape `map_node_bi` and the Stage-2 sugar `map_n_bi` share
a purpose (change N) but are distinct operations. Stage 1 rewrites the
base slots in place, cheaper when no chain exists yet. Stage 2 composes
a `ShapeLift` onto the chain — required after `.lift()`, and required
when an N-change should sit on top of earlier sugars.

### Where `wrap_init`'s second argument comes from

Every `wrap_*` user closure receives an `orig: &dyn Fn(...) -> ...`
parameter besides the node. That `orig` is structural, not stylistic:
it is the prior fold's corresponding phase, exposed as a value so the
sugar body can compose with what came before. The Lift family is, at
the type level, a triple of natural transformations between fold
algebras — and a phase mapper, by definition, takes the prior phase as
input and produces the new phase. See
[the type-level deep dive](../design/type_level.md#wrap_init-as-a-phase-mapper)
for the structural reasoning.

## Why traits, not inherent methods

Method names like `wrap_init` collide across domains (the Shared and
Local versions take different closure bounds). Two inherent `fn
wrap_init` on the same struct, parameterised differently by domain, do
not coexist under Rust's resolution. Each domain therefore exposes its
own trait — Rust selects the implementation matching the concrete
pipeline's domain marker. User code reads identically across Shared and
Local; only the bounds differ.

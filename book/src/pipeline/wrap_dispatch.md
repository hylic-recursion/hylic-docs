# Wrap dispatch — how Stage-2 sugars reach both Bases

`Stage2Pipeline<Base, L>` is one struct. Its sugar surface
(`Stage2SugarsShared` and `Stage2SugarsLocal`) is one trait per domain.
Yet its chain `L` operates over different node types depending on the
Base:

- `Stage2Pipeline<TreeishPipeline<…>, L>` — chain runs over the user's `N`.
- `Stage2Pipeline<SeedPipeline<…>, L>` — chain runs over `SeedNode<N>`,
  because [`SeedLift`](../concepts/lifts.md) prepends the synthetic
  EntryRoot at run time.

A user-facing closure types at `&N`. The chain expects `&<wrapped N>`.
Bridging the two is the job of `Wrap`.

## The trait

```rust
{{#include ../../../../hylic-pipeline/src/stage2/wrap/mod.rs:wrap_trait}}
```

Two impls:

- `Identity::Of<UN> = UN` — used by `TreeishPipeline`-rooted chains.
- `SeedWrap::Of<UN> = SeedNode<UN>` — used by `SeedPipeline`-rooted chains.

`Stage2Base` declares which `Wrap` a Base uses:

```rust
{{#include ../../../../hylic-pipeline/src/stage2/base.rs:stage2_base_trait}}
```

So in the type system: `<<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>`
is the chain's input N — equal to `UN` for treeish-rooted, equal to
`SeedNode<UN>` for seed-rooted. This two-hop projection appears verbatim
in every Stage-2 sugar's signature.

## Per-domain build subtraits

`Wrap` is type-only: it fixes a type family, not how to construct lifts.
The constructors live on per-domain subtraits:

```rust
{{#include ../../../../hylic-pipeline/src/stage2/wrap/shared.rs:wrap_shared_build_init_signature}}
```

(Plus one method per Stage-2 sugar; see
[`stage2/wrap/shared.rs`](../../../../hylic-pipeline/src/stage2/wrap/shared.rs)
for the full set, and
[`stage2/wrap/local.rs`](../../../../hylic-pipeline/src/stage2/wrap/local.rs)
for the Local mirror.)

The split is forced by the `Send + Sync` axis: `Shared` user closures must
be `Send + Sync` (Arc storage; parallel executors); `Local` must not require
it (`Rc` storage; supports non-Send captured state). `WrapShared`/`WrapLocal`
are how that single asymmetry is expressed without macros.

### Identity: pass-through

```rust
{{#include ../../../../hylic-pipeline/src/stage2/wrap/shared.rs:identity_build_wrap_init}}
```

User closure goes straight to `Shared::wrap_init_lift`. `Of<UN> = UN`, so
no adaptation is needed.

### SeedWrap: peel `Node`, pass `EntryRoot`

```rust
{{#include ../../../../hylic-pipeline/src/stage2/wrap/shared.rs:seedwrap_build_wrap_init}}
```

The user types `Fn(&UN, …) -> H`. The chain expects
`Fn(&SeedNode<UN>, …) -> H`. The body adapts: when the row is
`Node(n)`, call the user's closure with `&n`; when it's `EntryRoot`, call
through to the chain's `orig` continuation directly (the user closure has
nothing to do with the synthetic root).

The same pattern recurs for every N-aware sugar:
`build_filter_edges`, `build_memoize_by`, `build_wrap_visit`,
`build_map_n_bi`. Sugars without `&N` in their signature
(`wrap_accumulate`, `wrap_finalize`, `zipmap`, `map_r_bi`, `explain`) need
no peeling — both impls forward unchanged.

## How the sugar trait forwards

A representative `Stage2SugarsShared` body — the unified surface that
covers both Base shapes:

```rust
{{#include ../../../../hylic-pipeline/src/sugars/stage2_shared.rs:stage2_sugars_wrap_init}}
```

The body is one line. The surrounding `where` clauses repeat the projection
chain so Rust's solver can verify each junction; that's where the verbosity
sits. See [the type-level deep dive](../design/type_level.md) for why the
projection has to be spelled out symmetrically here.

## What the user sees

Nothing of the above. From the call site:

```text
seed_pipeline
    .lift()
    .wrap_init(|n: &N, orig| orig(n) + 1)   // typed at &N, not &SeedNode<N>
    .filter_edges(|n: &N| !is_excluded(n))
    .run_from_slice(&exec, &seeds, h0);
```

`Wrap` dispatch is invisible. The user picks a `Base`; the trait routes
through the right impl; closures stay typed at the user's `N`. Switching
`Base` shape — say, building the same chain over a `TreeishPipeline` —
costs no code at the sugar layer.

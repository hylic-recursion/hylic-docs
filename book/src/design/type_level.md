# The type-level landscape

This chapter is the design-level account of how the library's
functional concepts sit on top of Rust's type system. It assumes
familiarity with the [Lift](../concepts/lifts.md), [Wrap
dispatch](../pipeline/wrap_dispatch.md), and
[`SeedNode<N>`](../pipeline/seednode.md) chapters — and an interest in
*why* those shapes are what they are, beyond just *what* they do.

The library is a type-level functional kernel. Every transformation —
fold-phase rewrite, axis change, seed-closure — is a categorical
construct (natural transformation, type-level function, indexed family)
encoded in Rust's syntax. Where the encoding works smoothly the library
reads like ordinary Rust; where it doesn't, the friction shows up as
verbose projection chains, deliberate (`bi`-suffixed) bidirectionality,
or the per-domain split. Each of those is structural, not stylistic.

## GATs are higher-order functions

The first principle (lifted directly from
[Crichton, "GATs are HOFs"](https://willcrichton.net/notes/gats-are-hofs/)):
**a Generic Associated Type is a type-level function**. A GAT
`type Of<X>;` on a trait `T` is a function `X ↦ T::Of<X>`, where the
function's *body* is the impl's expansion.

In hylic, the canonical GAT is `Wrap::Of`:

```rust
{{#include ../../../../hylic-pipeline/src/stage2/wrap/mod.rs:wrap_trait}}
```

`Wrap` is a trait with a single GAT. Two impls give two functions:

```text
Identity::Of  : Type → Type   ≡  λUN. UN              -- identity
SeedWrap::Of  : Type → Type   ≡  λUN. SeedNode[UN]    -- one-tag wrap
```

These are not "associated types you happen to read off an impl"; they
are first-class type-level functions. The library uses them where a
Haskell library would use `f a` quantified over `f`, or a Scala 3
library would use `[F[_]]`. In Rust, the type lambda is encoded as a
trait with a GAT, and applied via the projection `<W as Wrap>::Of<UN>`.

## Lift as a triple of natural transformations

A `Fold<N, H, R>` has three phases:

```text
init        :  N → H
accumulate  :  (H, R) → ()       (mutates H)
finalize    :  H → R
```

A `Lift` transforms one fold algebra into another. Per the categorical
intuition, that is **a triple of natural transformations** — one per
phase. The general primitive `Shared::phases_lift` exposes exactly
that structure: it takes three *phase mappers*, each a function that
takes the prior fold's phase as a value and returns the new fold's
phase:

```text
init_mapper  :  (N → H)              → (N₂ → H₂)
acc_mapper   :  ((H, R) → ())        → ((H₂, R₂) → ())
fin_mapper   :  (H → R)              → (H₂ → R₂)
```

Compare with the `wrap_init` user closure:

```text
W : (N, prior_init) → H        -- curried form of the init_mapper
```

`prior_init` is the prior fold's `init` phase, currying `init_mapper`
into a friendlier shape: instead of "give me a function, get a
function," "give me a node and a function-on-nodes, get a value." The
user's `orig` argument is *not* a callback; it is the prior phase as a
first-class value, which composition needs as input. Drop the
parameter and you no longer have a phase mapper; you have a phase
*replacement*. Lift composition would stop being categorical.

This is the answer to "why does every wrap_* sugar take an `orig`
argument I sometimes don't use." The closure is a phase mapper. The
user not consulting `orig` is an identity-mapped composition — the
no-op natural transformation at that phase, which is structurally fine
but textually looks redundant.

## Why CPS in `Lift::apply`

A direct `apply` signature would return the transformed triple:

```text
apply : (Grow[Seed, N], Graph[N], Fold[N, H, R])
      → (Grow[Seed, N₂], Graph[N₂], Fold[N₂, H₂, R₂])
```

Each component is a domain-associated GAT and each axis is an
associated type of the lift. After three composed lifts the return
type involves three nested levels of associated-type projection, and
no single name in the language admits the result without spelling all
of them out. Rust's type inference does not span that distance.

CPS — continuation-passing style — sidesteps the unnameable.

```rust
{{#include ../../../../hylic/src/ops/lift/core.rs:lift_trait}}
```

`apply` takes a continuation `cont: impl FnOnce(triple) -> T`. The
continuation's return type `T` flows out unchanged. Inference threads
each intermediate triple through the continuation locally; nothing has
to be named at the top level. The composition reads as nested closure
calls; the executor's final `T` propagates outward through every
intermediate `apply`.

This is the same trick categorically: instead of returning a value of
some object in a category, take a hom-set element (a morphism out of
that object) and apply it. Rust's `impl Trait` argument is a way of
saying "a morphism from this object to *something*"; the *something*
is whatever shows up in the call chain.

## The two-hop projection

`Stage2Pipeline<Base, L>` is one struct. `Base` is `Stage2Base`:

```rust
{{#include ../../../../hylic-pipeline/src/stage2/base.rs:stage2_base_trait}}
```

The chain's input N is `<Base::Wrap as Wrap>::Of<UN>` — a *two-hop*
projection: first project `Base::Wrap` (a type), then project that type
through the `Wrap::Of` GAT at parameter `UN`. The full path:

```text
<<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>
   |__________ ___________|     |__ __|     |
              v                    v        v
        find Base from        find that     apply
        Self's Stage2Base     type's Wrap   Wrap::Of
        impl                  impl          at UN
```

For `Self::Base = TreeishPipeline<…>`, the chain unfolds to
`Identity::Of<UN> = UN`. For `Self::Base = SeedPipeline<…>`, it unfolds
to `SeedWrap::Of<UN> = SeedNode<UN>`. Both reduce; both are a single
projection chain; both work in every position the library uses
(method-return type, where-clause, GAT projection).

## What broke the first attempt

The original Phase-4 attempt had the sugar trait body call into a
*helper* trait whose return type was `<Self as Helper<UN>>::N`, while
the sugar method's declared return type was the full
`<<Base::Wrap as Wrap>::Of<UN>>` projection. Both reduced to the same
concrete type for any specific impl, but the *paths* through the type
system differed. The trait solver does not bridge two extensionally
equal but syntactically distinct projections inside a default body.

The fix: don't bridge. Have the sugar body call directly through the
projection that already names the chain's input N. The same projection
sits in the return-type slot, the where-clause, and the build-method
call. Rust's solver verifies syntactic equality in each position; no
reduction across distinct projection chains is ever required.

The mechanics — six probes isolating the typing positions — are
recorded in
[`KB/.plans/seed-pipeline-unification/pocs/`](../../../KB/.plans/seed-pipeline-unification/pocs/).

## Variance is structural, not friction

Every axis-change sugar with `_bi` in its name (`map_n_bi`,
`map_r_bi`, `map_node_bi`, `map_seed_bi`) takes a pair `(co, contra)`.
That is not Rust-specific verbosity. `N`, `H`, `R` are all *invariant*
in a fold algebra — `N` appears in `init`'s argument (contravariant)
and in `Graph<N>`'s child output (covariant); `R` appears in
`finalize`'s output and in `accumulate`'s child input. An invariant
type can only be transformed by an isomorphism; an isomorphism in
types is a pair of arrows.

Scala 3 needs the pair too. So does Haskell. The library's choice is
to expose the pair *explicitly*, named at the call site, rather than
hide it behind an `Iso` or `Bijection` typeclass. The "extra" closure
is the structural witness that the transform is an iso. In an
invariant world, that witness can't be elided.

## Send + Sync as a per-domain axis

Domains differ on closure storage and bound:

| Domain   | Closure cell      | Bound on user closures   |
|----------|-------------------|--------------------------|
| `Shared` | `Arc<dyn Fn …>`   | `Send + Sync + 'static`  |
| `Local`  | `Rc<dyn Fn …>`    | `'static`                |
| `Owned`  | `Box<dyn Fn …>`   | `'static` (and one-shot) |

The asymmetry is real: `Shared` parallel executors share the fold
across threads, so the fold's closures have to be `Send + Sync`;
`Local`'s `Rc` storage actively forbids `Send + Sync` on captured
state, allowing things like `Rc<RefCell<…>>` that the `Shared` form
rejects.

`Send + Sync` cannot be expressed as a uniform parameterisation of one
trait without macros (the bound is on a concrete closure type, not a
projection-able shape). The library's response is to split sugars and
the build dispatcher per domain: `WrapShared`/`WrapLocal`,
`Stage2SugarsShared`/`Stage2SugarsLocal`,
`SeedSugarsShared`/`SeedSugarsLocal`. The trait bodies read identically
line for line; only the bound differs. This is one of three
[accepted-debt items](../../../KB/hylic/legacy-plans/finishing-up/post-split-review/ACCEPTED-DEBT.md).

## Bounds at consumption, not construction

`Stage2Pipeline::then_lift` is *unconstrained* at the struct level —
pure construction:

```rust
{{#include ../../../../hylic-pipeline/src/stage2/primitives.rs:then_lift_primitive}}
```

A pipeline whose chain wouldn't actually `.run` is structurally
typeable. The compile-time check happens at consumption: `.run_*` and
the `TreeishSource` impl carry the chain-validity bounds. This is
deliberate. Construction is a builder; validity is a runner concern.
Imposing chain bounds at every `.then_lift` would force every
intermediate composition to be runnable, which loses the "construct
freely, validate at the consumption boundary" pattern that lets
chained sugars compose without each one having to fully type-prove
the chain so far.

## What this buys at runtime

Nothing has runtime overhead. The sugar trait monomorphises into a
chain of `ComposedLift` types; the type tree records every junction;
inlining flattens the chain into a single tree walk that produces one
`(treeish, fold)` pair. The executor never sees the chain — only its
collapsed result. `Wrap` dispatch resolves at compile time per
instantiation. The verbose projections in error messages are the price
of carrying that information through the type system; the compiled
binary has none of it.

## What remains as friction

Three things, all structural:

1. **No first-class higher-kinded types.** `Wrap::Of` is the closest
   approximation. Verbose two-hop projections are the cost.

2. **No type-level pattern matching (no Scala 3 *match types*).** Rust
   cannot decompose `SeedNode<UN>` into `UN` at the type level. If a
   trait's bound says `L::N2 = SeedNode<UN>`, `UN` must be supplied
   from elsewhere (e.g., a closure argument's inferred type) — Rust
   will not invert the constructor.

3. **No macros.** The Shared/Local mirror could be one file with
   per-domain bound sugar, but the codebase declines macro-generated
   trait bodies. The duplication is documented and accepted.

What is *not* friction: bidirectional axis transforms (universal),
`orig` callbacks in `wrap_init` (structural natural-transformation
shape), CPS in `apply` (only way to thread unnameable returns). Those
are the right shapes; Rust just exposes them at the level the
abstraction needs.

## Wrap_init as a phase mapper

The `wrap_init` family deserves a closer read because the user closure
*looks* like a callback-with-fallthrough but is actually a curried
phase mapper. The sugar's user signature is

```text
W : (N, prior_init: dyn Fn N → H) → H
```

curried from the underlying

```text
init_mapper : (N → H) → (N → H)
            ≡ Fn(prior_init) → Fn(N → H)        -- uncurried
            ≡ Fn(N, prior_init) → H              -- curried; same content
```

The sugar's body in the library realises this:

```rust
{{#include ../../../../hylic/src/domain/shared/shape_lifts/fold_sugars.rs:shared_wrap_init_lift_body}}
```

Reading the body: take the user wrapper `w`, return a function from
the prior init `old` to a new init that, on each `n`, calls
`w(n, &*old)`. The closure-passed-in is the prior phase, exposed as a
value. That is the structural definition of a phase mapper. The
"intercept" framing is incidental; the categorical content is
*compose this layer's natural transformation with the prior layer's*.

The same shape recurs, with appropriate types, in
`wrap_accumulate_lift` and `wrap_finalize_lift`. The general primitive
they all collapse to is `Shared::phases_lift` — three phase mappers,
one per phase, each taking the prior phase and producing the next.

## Reading list

- [Crichton, "GATs are HOFs"](https://willcrichton.net/notes/gats-are-hofs/)
  for the GAT framing.
- [`KB/hylic/legacy-plans/seed-pipeline-transforms/PRINCIPLES.md`](../../../KB/hylic/legacy-plans/seed-pipeline-transforms/PRINCIPLES.md)
  for the "every specific transform should derive from a general one"
  principle.
- [Lifts](../concepts/lifts.md) for the trait shape and the four atoms.
- [Wrap dispatch](../pipeline/wrap_dispatch.md) for the surface where
  the type-level machinery lands at the user's call site.
- [`pocs/FINDINGS.md`](../../../KB/.plans/seed-pipeline-unification/pocs/FINDINGS.md)
  for the empirical record of which projection-chain positions Rust's
  current solver does and doesn't handle.

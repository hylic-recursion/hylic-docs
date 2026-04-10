# Theory notes

hylic implements patterns from the theory of recursion schemes, adapted
for Rust's type system. This page maps hylic's types to their formal
names for readers who want to connect to the literature.

## Catamorphism (fold)

A catamorphism consumes a recursive structure bottom-up. hylic's `Fold<N, H, R>`
is a monoidal catamorphism — the algebra is decomposed into three phases
(init, accumulate, finalize) through an intermediate heap type `H`.

The standard formulation is a single function `F<R> → R` where `F` is the
base functor. hylic's three-phase decomposition enables independent
transformation of each phase via `wrap_init`, `wrap_accumulate`, `wrap_finalize`.

## Anamorphism (unfold)

An anamorphism builds a recursive structure from a seed. hylic's `SeedGraph`
is a coalgebra: given a seed, produce one layer of structure (the node and
its child seeds). The `grow` function is the coalgebra.

## Hylomorphism (unfold then fold)

When a `Treeish` is backed by lazy child discovery (via `SeedGraph`), the
catamorphism and anamorphism fuse — the tree is never fully materialized.
This is a hylomorphism, and it's what `Exec::run()` performs when the
graph is lazy. The [Funnel executor](../funnel/overview.md) parallelizes
this pattern using [CPS](../funnel/cps_walk.md) (continuation-passing
style) and [defunctionalized tasks](../funnel/continuations.md).

## Histomorphism (fold with history)

The `Explainer` wraps a fold to record the full computation trace at every
node — the initial heap, each child result folded in, and the final result.
This corresponds to a histomorphism, which is a catamorphism where each
node has access to the full computation history of its subtree.

In recursion-scheme terms, the Explainer's output (`ExplainerResult`) is
analogous to the cofree comonad annotation. The Explainer is expressed
as a Lift — it transforms `Fold<N, H, R>` into
`Fold<N, ExplainerHeap, ExplainerResult>` and `unwrap` extracts the
original `R`.

## Natural transformation (Lift)

The `LiftOps<N, R, N2>` trait represents a natural transformation
between two F-algebras. It maps the carrier types of one algebra
to another through two GATs (`LiftedH<H>`, `LiftedR<H>`) while
preserving the fold's computational structure. The `unwrap` function
recovers the original algebra's result from the lifted computation.

Concrete lifts implement the trait as structs: the Explainer lifts
into a trace-recording domain (histomorphism), the SeedLift lifts
into an `Either<Seed, Node>` domain for seed-based graph construction.

The key property: `lift::run_lifted(&exec, &lift, &fold, &graph, &root)`
produces the same `R` as `exec.run(&fold, &graph, &root)` — the lift
is transparent to the result.

## Externalized tree structure

Classical recursion schemes encode the recursive structure in the type
system via fixed points of functors (`Fix(F)`). hylic externalizes it as
a runtime function (`Treeish<N>` = `Fn(&N, &mut dyn FnMut(&N))`). This
trades compile-time structural guarantees for runtime flexibility: the
same fold works with any tree shape without redefining types.

## Operations traits and domain abstraction

`FoldOps<N, H, R>` and `TreeOps<N>` abstract the fold and graph
operations from their storage. The standard types (`Fold`, `Treeish`)
store closures behind Arc (the Shared domain). Alternative
implementations can use Rc (Local), Box (Owned), or concrete structs
(zero-boxing). The executor's recursion engine takes `&impl FoldOps +
&impl TreeOps` — fully generic over the storage, monomorphized to
zero overhead for concrete types.

This is a form of defunctionalization: the operations traits are the
abstract interface; the domain-specific types are the concrete
representations. The `Domain` trait with GATs maps the marker type
(Shared, Local, Owned) to the concrete types — a type-level function
from boxing strategy to implementation.

## Domain as functor

The `Domain` trait is a type-level functor: it maps a node type `N` to
a family of concrete types (`Fold<H, R>`, `Treeish`). In category
theory terms, each domain is a functor from the category of node types
to the category of fold/graph implementations:

```
Domain<N> : N ↦ (Fold<H, R>, Treeish)
```

The GAT formulation makes this explicit:

```rust
trait Domain<N> {
    type Fold<H, R>: FoldOps<N, H, R>;
    type Treeish: TreeOps<N>;
}
```

`Shared`, `Local`, and `Owned` are three different functors with the
same signature — they agree on the interface (FoldOps, TreeOps) but
disagree on the representation (Arc, Rc, Box). Code that is generic
over `D: Domain<N>` is a natural transformation: it works uniformly
across all three functors.

The executor's domain parameter (`Exec<D, S>`) selects which functor
to apply. The inherent method trick exploits this: D is fixed by the
executor const's type, so the compiler resolves the GATs statically.
No runtime dispatch over the domain — the functor application is fully
monomorphized.

## SyncRef as proof witness

`SyncRef<'a, T>` is an unsafe wrapper that asserts `Send + Sync` for
a borrowed reference. It serves as a proof witness for a specific
safety argument: data borrowed within a scoped thread pool outlives all
workers, so shared access is safe even for types that are normally
`!Sync`.

The formal structure:

1. **Premise**: `WorkPool::with(spec, |pool| { ... })` guarantees all
   worker threads join before the closure returns (via
   `std::thread::scope`).
2. **Invariant**: Within the scope, any `&T` with lifetime `'scope`
   outlives all tasks submitted to the pool.
3. **Obligation**: Workers must not mutate the borrowed data or clone
   the inner wrapper (e.g., no `Rc::clone` through the reference).
4. **Witness**: `SyncRef(&data)` encodes that the caller has verified
   premises 1–3. The `unsafe impl Send + Sync` is the proof
   discharge.

This pattern makes domain-generic parallel execution possible.
Without SyncRef, `&Rc<dyn Fn>` is `!Send` (because `Rc` is `!Sync`),
blocking any cross-thread sharing. SyncRef bypasses this by asserting
that the specific usage pattern (read-only borrows within a scoped
pool) is safe — the Rc refcount is never touched by workers.

The safety argument is local to the pool boundary: SyncRef is created
inside the recursion engine and never escapes it. Outside the pool,
normal Rust lifetime and Send/Sync rules apply unchanged.

The funnel executor applies the same scoped-lifetime argument to its
`RootCell` — the fold's terminal result cell lives on `run_fold`'s
stack and is accessed through a raw pointer (`*const RootCell<R>`)
carried by `Cont::Root`. The scoped pool guarantees the pointer is
valid for all workers. No Arc, no heap allocation.

## ConstructFold: domain-generic fold construction

`ConstructFold<N>` is a type-level function from a domain marker to a
fold constructor. Given three closures (init, accumulate, finalize),
it produces a `D::Fold<H, R>` — wrapping in Arc for Shared, Rc for
Local. This enables lifts to construct domain-appropriate folds without
knowing the concrete domain at the generic code level.

The challenge: Shared's fold requires `Send + Sync` closures, Local's
does not. A single trait method can't express varying bounds per impl.
The solution: `make_fold` is `unsafe fn` with a documented contract —
the Shared impl uses `AssertSend` (an unsafe Send+Sync wrapper) to
bridge the gap. The safety of this bridge rests on the observation
that closures passed to `ConstructFold<Shared>` capture Shared-domain
data (Arc-based), which IS Send+Sync.

## Data tree decoupling

The parallel lifts (ParLazy, ParEager) decouple the computation's
*data* from the fold's *operations*. Phase 1 builds a tree of pure
data nodes (heap values + child handles). Phase 2 applies the fold's
accumulate and finalize through an external reference — SyncRef for
ParLazy (scoped borrow), FoldPtr for ParEager (lifetime-erased raw
pointer).

This decoupling is what makes domain-generic parallel lifts possible.
Without it, each node would capture domain-specific closures (Arc for
Shared, Rc for Local), and Rc closures can't cross thread boundaries.
By separating data from operations, the data nodes are domain-agnostic
and the fold reference is provided at evaluation time through an
appropriate unsafe primitive.

## Further reading

- Meijer, Fokkinga, Paterson. *Functional Programming with Bananas, Lenses, Envelopes and Barbed Wire.* (1991) — the original recursion schemes paper.
- Milewski. *Monoidal Catamorphisms.* (2020) — the decomposition hylic's fold uses.
- Kmett. [recursion-schemes](https://hackage.haskell.org/package/recursion-schemes) — Haskell reference implementation.
- Malick. [recursion.wtf](https://recursion.wtf/) — practical recursion schemes in Rust.

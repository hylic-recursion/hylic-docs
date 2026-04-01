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
transformation of each phase via `map_init`, `map_accumulate`, `map_finalize`.

## Anamorphism (unfold)

An anamorphism builds a recursive structure from a seed. hylic's `SeedGraph`
is a coalgebra: given a seed, produce one layer of structure (the node and
its child seeds). The `grow` function is the coalgebra.

## Hylomorphism (unfold then fold)

When a `Treeish` is backed by lazy child discovery (via `SeedGraph`), the
catamorphism and anamorphism fuse — the tree is never fully materialized.
This is a hylomorphism, and it's what `Exec::run()` performs when the
graph is lazy.

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

A `Lift<N, H, R, N2, H2, R2>` is a natural transformation between two
F-algebras. It maps the carrier types of one algebra to another while
preserving the fold's computational structure. The `unwrap` function
recovers the original algebra's result from the lifted computation.

hylic uses Lifts for:
- **Explainer**: lift into a trace-recording domain (histomorphism)
- **ParLazy**: lift into a deferred-evaluation domain (`ParRef<R>`)
- **ParEager**: lift into an extracted-heap domain (`EagerNode<H>`)

The key property: `exec.run_lifted(&lift, &fold, &graph, &root)` produces
the same `R` as `exec.run(&fold, &graph, &root)` — the Lift is transparent.

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

## Further reading

- Meijer, Fokkinga, Paterson. *Functional Programming with Bananas, Lenses, Envelopes and Barbed Wire.* (1991) — the original recursion schemes paper.
- Milewski. *Monoidal Catamorphisms.* (2020) — the decomposition hylic's fold uses.
- Kmett. [recursion-schemes](https://hackage.haskell.org/package/recursion-schemes) — Haskell reference implementation.
- Malick. [recursion.wtf](https://recursion.wtf/) — practical recursion schemes in Rust.

# Theory notes

hylic implements patterns from the theory of recursion schemes,
adapted for Rust's type system. This page maps hylic's types to
their formal names.

## Catamorphism

A catamorphism is a bottom-up fold over a recursive structure. The
algebra is `F R → R` — given one layer of structure with children
already folded to `R`, produce `R`. The carrier type `R` is the
result at every subtree.

hylic factors this algebra into three steps through an intermediate
type `H`:

```
F R → R  =  init(&N) → H, accumulate(&mut H, &R) per child, finalize(&H) → R
```

`H` is mutable working state internal to each node. `R` is the
immutable result that flows between nodes. The bracket
(init opens `H`, finalize closes to `R`) makes the invariant
boundary explicit. See
[The N-H-R algebra factorization](milewski.md) for the comparison
with Milewski's monoidal decomposition and the equivalence under
associative `⊕`.

## Hylomorphism

When the tree structure is not materialized but discovered on demand
(via a `Treeish` backed by lazy child discovery), the unfold
(anamorphism) and fold (catamorphism) fuse — the tree exists only as
a call stack, never as a data structure. This is a hylomorphism.

In hylic, every `Exec::run()` call is a hylomorphism: the executor
receives a coalgebra (`Treeish<N>`, which produces children on
demand) and an algebra (`FoldOps<N, H, R>`, which consumes them),
and fuses both in a single recursive pass. `(N, Treeish<N>)` is
hylic's runtime equivalent of the type-level `Fix (f a)` — the pair
describes a root and a way to get children, recursively, without
materializing the tree.

The [Funnel executor](../funnel/overview.md) parallelizes the
hylomorphism using [CPS](../funnel/cps_walk.md) and
[defunctionalized tasks](../funnel/continuations.md).

## Anamorphism (seed-based discovery)

An anamorphism builds recursive structure from a seed.
[`SeedPipeline`](../pipeline/seed.md) encapsulates this:
given a seed edge function (`Edgy<N, Seed>`) and a grow function
(`Fn(&Seed) → N`), it constructs the treeish by composing
`seeds_from_node.map(grow)` and handles the entry transition.
Internally, `SeedLift` implements `Lift` to express the
`LiftedNode<Seed, N>` indirection as a fold transformation.

## Histomorphism (fold with history)

The `Explainer` records the full computation trace at every node —
initial heap, each child result folded in, and the final result.
This corresponds to a histomorphism: a catamorphism where each node
has access to the full computation history of its subtree.

The Explainer's output (`ExplainerResult`) is analogous to the
cofree comonad annotation. It is expressed as a
[Lift](../concepts/lifts.md) — a fold transformation that changes the
carrier types (`H → ExplainerHeap`, `R → ExplainerResult`). The
original `R` is accessible via `ExplainerResult::orig_result`.

## Algebra morphism (Lift)

`Lift<N, N2>` maps one fold algebra into another. It
transforms the carrier types through two GATs (`MapH<H, R>`,
`MapR<H, R>`) and can change the node type (`N → N2`) by extending
the tree structure with new constructors.

The SeedLift extends the tree with relay and entry constructors
(`LiftedNode<Seed, N>`: Entry, Seed, Node) — seed nodes pass their
single child's result through unchanged. The Explainer enriches the heap with trace data
without changing the node type. Both are algebra morphisms: they
transform the `F R → R` algebra into a different `F' R' → R'`
algebra over a richer domain.

`lift::run_lifted` applies the three trait methods (lift_treeish,
lift_fold, lift_root), runs the lifted computation, and returns
`MapR<H, R>`.

## Externalized tree structure

Classical recursion schemes encode tree structure via fixed points
of functors (`Fix F`). The functor `F` defines one layer of shape
(leaf, binary node, n-ary node), and `Fix F` is the recursive
nesting.

hylic externalizes this as a runtime function: `Treeish<N>` is
`Fn(&N, &mut dyn FnMut(&N))`. The node type `N` carries identity,
not structure — the same `N` can be traversed by different treeish
functions, and the same fold works with any tree shape. This
trades compile-time structural guarantees for the orthogonal
decomposition of fold, graph, and executor.

The pair `(N, Treeish<N>)` corresponds to a coalgebra `N → F N` —
the treeish IS the coalgebra, producing one layer of children on
demand. Combined with the fold algebra, the executor performs a
fused hylomorphism.

## Operations traits and domain abstraction

`FoldOps<N, H, R>` and `TreeOps<N>` abstract the fold and graph
operations from their storage. The standard types (`Fold`, `Treeish`)
store closures behind Arc (the Shared domain). Alternative
implementations can use Rc (Local), Box (Owned), or concrete structs
(zero-boxing). The executor's recursion engine takes `&impl FoldOps +
&impl TreeOps` — fully generic over the storage, monomorphized to
zero overhead for concrete types.

The `Domain` trait with GATs maps the marker type (Shared, Local,
Owned) to concrete fold types. Graph types are domain-independent —
always Arc-based, always `Send + Sync`. The domain controls only
how fold closures are stored.

## Further reading

- Meijer, Fokkinga, Paterson. *Functional Programming with Bananas, Lenses, Envelopes and Barbed Wire.* (1991) — the original recursion schemes paper.
- Milewski. [Monoidal Catamorphisms](https://bartoszmilewski.com/2020/06/15/monoidal-catamorphisms/) (2020) — a different algebra factorization. See [comparison](milewski.md).
- Gonzalez. [foldl](https://hackage.haskell.org/package/foldl) — the left-fold-with-extraction pattern.
- Kmett. [recursion-schemes](https://hackage.haskell.org/package/recursion-schemes) — Haskell reference implementation.
- Malick. [recursion.wtf](https://recursion.wtf/) — practical recursion schemes in Rust.

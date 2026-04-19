# The N-H-R algebra factorization

A catamorphism's algebra collapses one layer of recursive structure.
The standard formulation is a single morphism `F R → R`. Both hylic
and Milewski's [monoidal catamorphism](https://bartoszmilewski.com/2020/06/15/monoidal-catamorphisms/)
factor this morphism into composable steps. They factor it
differently. This page establishes the precise relationship between
the two and shows when one can be derived from the other.

## The two formulations

| | hylic | Milewski |
|---|---|---|
| **Extract** | `init: &N → H` | `s: a → m` (scatter) |
| **Combine** | `acc: &mut H, &R` | `⊕: m × m → m` (monoid) |
| **Output** | `fin: &H → R` (every node) | `g: m → b` (root only) |
| **Working type** | `H` (unconstrained) | `m` (associative, with identity) |
| **Carrier** | `R` | `m` |

In hylic, the carrier is `R`. Every subtree produces `R`. In
Milewski, the carrier is `m`. Every subtree produces `m`, and a
separate function `g` converts to the output type `b` once at the
root.

## The bracket

At each node, `init` opens mutable working state `H`, `accumulate`
folds each child's `R` into it, and `finalize` closes it to `R`.
The heap `H` never crosses node boundaries. Only `R` flows between
nodes.

```dot process
digraph bracket {
    rankdir=TB;
    compound=true;
    node [fontname="sans-serif", fontsize=9, shape=box, style="rounded,filled"];
    edge [fontname="sans-serif", fontsize=8];

    subgraph cluster_child1 {
        label="child 1"; labeljust=l; style="rounded,filled";
        fillcolor="#f1f8e9"; color="#a5d6a7"; fontname="sans-serif"; fontsize=9;
        c1_init [label="init(N₁) → H", fillcolor="#c8e6c9"];
        c1_fin  [label="fin(H) → R", fillcolor="#bbdefb"];
        c1_init -> c1_fin [label="(leaf)"];
    }

    subgraph cluster_child2 {
        label="child 2"; labeljust=l; style="rounded,filled";
        fillcolor="#f1f8e9"; color="#a5d6a7"; fontname="sans-serif"; fontsize=9;
        c2_init [label="init(N₂) → H", fillcolor="#c8e6c9"];
        c2_fin  [label="fin(H) → R", fillcolor="#bbdefb"];
        c2_init -> c2_fin [label="(leaf)"];
    }

    subgraph cluster_parent {
        label="parent"; labeljust=l; style="rounded,filled";
        fillcolor="#f1f8e9"; color="#a5d6a7"; fontname="sans-serif"; fontsize=9;
        p_init [label="init(N₀) → H", fillcolor="#c8e6c9"];
        p_acc1 [label="acc(&mut H, &R₁)", fillcolor="#c8e6c9"];
        p_acc2 [label="acc(&mut H, &R₂)", fillcolor="#c8e6c9"];
        p_fin  [label="fin(H) → R", fillcolor="#bbdefb"];
        p_init -> p_acc1 -> p_acc2 -> p_fin;
    }

    c1_fin -> p_acc1 [label="R₁", color="#1565c0", fontcolor="#1565c0"];
    c2_fin -> p_acc2 [label="R₂", color="#1565c0", fontcolor="#1565c0"];
}
```

Green is `H`-world (mutable working state). Blue is `R` (immutable
result). The green-to-blue transition at each node is the finalize
step, the bracket closing.

The node type `N` seeds the heap but is not part of the algebra. It
is the node's identity; the recursive structure lives in
[`Treeish<N>`](../concepts/separation.md), not in `N`. The pair
`(N, Treeish<N>)` is hylic's runtime equivalent of Milewski's
type-level `Fix (f a)`.

The bracket separates mutable working state from immutable results.
`H` can be a growable `Vec` while `R` is a frozen `Arc<[T]>`, for
example. Without the bracket, the user would either accumulate into
Arc (expensive reallocation on every push) or return Vec as the
result (wrong invariant for the parent, which expects immutable
data). The Rust type system reinforces this: `&mut H` is
single-owner and never shared, while `R` can be `Send` and cross
thread boundaries. The [Funnel executor](../funnel/overview.md)
exploits this directly. `R` values are delivered across threads via
[slot delivery](../funnel/accumulation.md); `H` stays on the
sweeping thread. For single-child nodes, the bracket is carried as
a [direct continuation](../funnel/continuations.md) with no
allocation and no atomic. Each [phase can be wrapped
independently](../cookbook/transformations.md) via `wrap_init`,
`wrap_accumulate`, `wrap_finalize`.

## The monoidal form

In Milewski's decomposition, the working type `m` is a monoid
(associative binary operation `⊕` with identity `ε`). A `Fold(s, g)`
pairs a scatter function `s: a → m` with a gather function
`g: m → b`. An `MAlgebra` provides the structural combination rule,
combining one layer of the functor using only `⊕` and `ε`.

The catamorphism `cat malg (Fold s g) = g ∘ cata (malg ∘ bimap s id)`
produces `m` at every node. `g` converts to `b` once at the root.

```dot process
digraph monoidal {
    rankdir=TB;
    compound=true;
    node [fontname="sans-serif", fontsize=9, shape=box, style="rounded,filled"];
    edge [fontname="sans-serif", fontsize=8];

    subgraph cluster_child1m {
        label="child 1"; labeljust=l; style="rounded,filled";
        fillcolor="#e8f5e9"; color="#a5d6a7"; fontname="sans-serif"; fontsize=9;
        c1m_s [label="s(a₁) → m", fillcolor="#c8e6c9"];
    }

    subgraph cluster_child2m {
        label="child 2"; labeljust=l; style="rounded,filled";
        fillcolor="#e8f5e9"; color="#a5d6a7"; fontname="sans-serif"; fontsize=9;
        c2m_s [label="s(a₂) → m", fillcolor="#c8e6c9"];
    }

    subgraph cluster_parentm {
        label="parent"; labeljust=l; style="rounded,filled";
        fillcolor="#e8f5e9"; color="#a5d6a7"; fontname="sans-serif"; fontsize=9;
        pm_s    [label="s(a₀) → m", fillcolor="#c8e6c9"];
        pm_op1  [label="m ⊕ m₁", fillcolor="#c8e6c9"];
        pm_op2  [label="m ⊕ m₂", fillcolor="#c8e6c9"];
        pm_s -> pm_op1 -> pm_op2;
    }

    c1m_s -> pm_op1 [label="m₁", color="#2e7d32", fontcolor="#2e7d32"];
    c2m_s -> pm_op2 [label="m₂", color="#2e7d32", fontcolor="#2e7d32"];

    g [label="g(m) → b", fillcolor="#bbdefb"];
    pm_op2 -> g [label="m", color="#2e7d32", fontcolor="#2e7d32"];
}
```

Compare the two diagrams. In the bracket form, every node has a
green-to-blue transition (per-node finalize). In the monoidal form,
green `m` flows uniformly and the single blue step occurs at the
root.

## Relationship

**Claim.** Milewski's monoidal catamorphism is a special case of
hylic's N-H-R fold.

**Proof.** Given a Milewski fold with monoid `(m, ⊕, ε)`, scatter
`s: a → m`, and gather `g: m → b`, construct the hylic fold:

```
H = R = m,   init = s,   acc = ⊕,   fin = identity
```

At each node, hylic computes `acc(acc(init(n), r₁), r₂)`
= `s(n) ⊕ r₁ ⊕ r₂`. This is the value Milewski's catamorphism
produces at every node. The user applies `g` to the root result to
obtain `b`. ∎

**Conditions for the converse.** A hylic fold is expressible as a
Milewski monoidal catamorphism when:

- `H = R` and `fin = identity`
- `acc` is a monoid (associative with identity element)

These make `(H, acc, ε)` a monoid. The correspondence is then
`m = H`, `s = init`, `⊕ = acc`, `g = identity`.

Without these conditions, hylic's fold is strictly more general. It
admits non-associative accumulation and distinct working/result
types.

## Examples

Folds that satisfy the monoid conditions:

- **Sum.** `H = R = u64`, `acc = +`, `fin = id`. Addition with
  identity 0.
- **Extend.** `H = R = Vec<T>`, `acc = extend`, `fin = clone`.
  Concatenation with identity `vec![]`. The
  [filesystem summary](../cookbook/filesystem_summary.md) uses this.
- **Union.** `H = R = HashSet<K>`, `acc = union`, `fin = clone`.
  Associative and commutative.

Folds that do not:

- **Child count.** `acc((s,c), r) = (s+r, c+1)`. The count tracks
  immediate children, not descendants. Not associative:
  `(h₁⊕h₂)⊕h₃` yields `c+2` while `h₁⊕(h₂⊕h₃)` yields `c+1`.
- **Bracketed formatting.** `fin(h) = format!("[{}]", h)`. Here
  `H ≠ R` and regrouping changes the nesting: `[a[b]][c] ≠
  [a][b[c]]`.

## Associativity and parallel accumulation

A monoid's associativity allows the executor to contract adjacent
sibling results in any grouping. If children b and c have completed
but a has not, `b ⊕ c` can proceed without waiting for a. When a
eventually completes, it combines with the already-contracted result.
For n children, this reduces the accumulation depth from O(n) to
O(log n).

hylic's [Funnel executor](../funnel/overview.md) does not perform
this contraction. It parallelizes subtree computation (children run
concurrently on different workers) and accumulates their results
left-to-right as the [sweep cursor](../funnel/accumulation.md)
advances. This is a design choice: sequential accumulation enables
progressive memory freeing, where each child's `R` is consumed and
dropped as the cursor passes. It also means the executor imposes no
algebraic requirements on `acc`. It is up to the user to supply an
appropriate accumulate function, and up to the executor to decide
how results are folded into `H`.

A [lift](../guides/lifts.md) can recover O(log n) depth when
needed: by transforming the tree structure to insert balanced
reduction nodes, the contraction becomes a property of the tree
shape rather than the algebra.

## The general structure

In algebraic terms, `acc: H × R → H` is an action of `R` on `H`.
When `H = R` and `acc` is a monoid, this is a monoid acting on
itself, which is Milewski's formulation. In general, it is an
R-module: `R` acts on a distinct type `H` through `acc`, with
`fin: H → R` as the projection. A monoid is a module over itself;
a module is not necessarily a monoid.

hylic's API does not distinguish between these cases. The user
writes `init`, `acc`, `fin`. The executor runs them with sequential
accumulation and parallel subtree computation via
[CPS work-stealing](../funnel/cps_walk.md).

## Composability

hylic's fold combinators
([`product`](../cookbook/filesystem_summary.md),
[`map`](../guides/fold.md), [`zipmap`](../guides/fold.md),
[`wrap_*`](../cookbook/transformations.md)) and graph combinators
([`filter`](../guides/graph.md),
[`memoize`](../guides/graph.md),
[`contramap`](../guides/graph.md)) achieve the same practical
composability as Milewski's `Functor`/`Applicative` on `Fold`.
[Lifts](../guides/lifts.md) transform both fold and treeish in
sync, changing the carrier types through GATs. The
[SeedPipeline](../guides/seed_pipeline.md) uses a lift internally
to bridge coalgebra and algebra when they speak different types.

## Bridging coalgebra and algebra: SeedPipeline

A hylomorphism fuses a coalgebra (produce children) with an algebra
(fold results). Both operate on the same type `N`. In practice, the
dependency structure often speaks a different type. A module
resolver starts with module names (seeds), not parsed modules
(nodes). A `grow` function resolves one into the other.

The user provides:

```
grow:            Fn(&Seed) → N           resolve a reference
seeds_from_node: N → Seed*              a node's dependency references
fold:            FoldOps<N, H, R>        the algebra, defined over N
```

In hylic, `N → Seed*` is `Edgy<N, Seed>`, the general edge
function. `N → N*` is `Treeish<N>`, the special case where node and
edge types match.

The coalgebra produces `Seed`. The algebra consumes `N`. The
morphism `grow: Seed → N` bridges them.
[`SeedPipeline`](../guides/seed_pipeline.md) reconciles this
through two combinator chains.

**Chain 1: coalgebra composition.** Close `N → Seed*` into
`N → N*` via `.map(grow)`:

```
seeds_from_node: Edgy<N, Seed>             N → Seed*
    .map(grow)                             Seed → N
= treeish:       Edgy<N, N>               N → N*  (= Treeish<N>)
```

In code (`SeedPipeline::new`):

```rust
{{#include ../../../../hylic/src/cata/seed_lift/pipeline_run.rs:treeish_from_seeds}}
```

The underlying combinator (`map_edges`):

```rust
{{#include ../../../../hylic/src/graph/combinators.rs:map_edges}}
```

**Chain 2: entry lifting.** The `SeedLift` constructs a
`Treeish<LiftedNode<Seed, N>>` that dispatches per variant:
`Node(n)` visits the original treeish (wrapping children as `Node`),
`Seed(s)` produces one child `Node(grow(s))`, and `Entry` fans out
the entry seeds as `Seed` children.

In code (`SeedLift::lift_treeish`):

```rust
{{#include ../../../../hylic/src/cata/seed_lift/lift.rs:lift_treeish}}
```

`Node(n)` delegates to the inner treeish. `Seed(s)` grows and
produces a single `Node` child. `Entry` has no children of its
own in the treeish — its children come from the entry seeds
provided at run time.

```dot process
digraph seed_bridge {
    rankdir=TB;
    compound=true;
    node [fontname="sans-serif", fontsize=9, shape=box, style="rounded,filled"];
    edge [fontname="sans-serif", fontsize=8];

    subgraph cluster_user {
        label="user provides"; labeljust=l; style="rounded,filled";
        fillcolor="#fafafa"; color=grey80; fontname="sans-serif"; fontsize=9;

        sfn [label="seeds_from_node\nN → Seed*", fillcolor="#fff3cd"];
        grow [label="grow\nFn(&Seed) → N", fillcolor="#fff3cd"];
        fold_u [label="fold\nFoldOps<N, H, R>", fillcolor="#fff3cd"];
    }

    subgraph cluster_compose {
        label="pipeline constructs"; labeljust=l; style="rounded,filled";
        fillcolor="#fafafa"; color=grey80; fontname="sans-serif"; fontsize=9;

        t [label="Treeish<N>\nseeds_from_node.map(grow)", fillcolor="#c8e6c9"];
    }

    sfn -> t [label=".map(grow)"];
    grow -> t [style=dashed];

    subgraph cluster_lift {
        label="SeedLift extends"; labeljust=l; style="rounded,filled";
        fillcolor="#fafafa"; color=grey80; fontname="sans-serif"; fontsize=9;

        lt [label="Treeish<LiftedNode<Seed, N>>\nper-variant dispatch", fillcolor="#dcedc8"];
        lf [label="Fold<LiftedNode<Seed, N>,\n     LiftedHeap<H,R>, R>", fillcolor="#dcedc8"];
    }

    t -> lt [label="lift_treeish"];
    fold_u -> lf [label="lift_fold"];

    subgraph cluster_entry {
        label="entry"; labeljust=l; style="rounded,filled";
        fillcolor="#fafafa"; color=grey80; fontname="sans-serif"; fontsize=9;

        entry_point [label="Entry", fillcolor="#e1bee7"];
        entry_seed [label="Seed(s)", fillcolor="#ffccbc"];
        entry_node [label="Node(grow(s))", fillcolor="#c8e6c9"];
        entry_rest [label="original treeish + fold\ndrive all further traversal", fillcolor="#c8e6c9"];
    }

    entry_point -> entry_seed [label="entry seeds"];
    entry_seed -> entry_node [label="grow"];
    entry_node -> entry_rest [label="converges"];

    lt -> entry_point [label="exec.run", style=dashed];
}
```

After the `Entry → Seed → Node` transition, the original coalgebra
and algebra drive all further recursion. The `LiftedNode` type, the
`LiftedHeap`, and the composed treeish are internal to the pipeline.

Entry seeds are supplied at run time via `Edgy<(), Seed>` passed to
`pipeline.run(exec, entry_seeds, initial_heap)`, or via
`pipeline.run_from_slice(exec, &[seed1, seed2], initial_heap)`.
The pipeline itself stores no entry concerns — only `grow`,
`seeds_from_node`, and the fold.

## Further reading

- Milewski. [Monoidal Catamorphisms](https://bartoszmilewski.com/2020/06/15/monoidal-catamorphisms/) (2020).
- Gonzalez. [foldl](https://hackage.haskell.org/package/foldl) — the left-fold-with-extraction type.
- Meijer, Fokkinga, Paterson. *Functional Programming with Bananas, Lenses, Envelopes and Barbed Wire* (1991).

# The recursive pattern

Every recursive tree computation does the same thing. hylic
makes that pattern explicit, separates its parts, and lets you
transform each part independently.

## One function

This is the entire computation, from `exec.rs`:

```rust
{{#include ../../../../hylic/src/cata/exec/variant/fused/mod.rs:run_inner}}
```

Read it carefully. At each node:

1. **init** — create a heap `H` from the node
2. **visit children** — for each child, recurse and accumulate the result
3. **finalize** — produce the node's result `R` from the heap

That's it. Every tree fold — fibonacci, dependency resolution,
filesystem aggregation, AST evaluation — is this function with
different `init`, `accumulate`, `finalize`, and different child
structure.

```dot process
digraph {
    rankdir=TB;
    node [shape=box, style="rounded,filled", fillcolor="#f5f5f5", fontname="monospace", fontsize=11];
    edge [fontname="sans-serif", fontsize=10];

    subgraph cluster_node {
        label="One node's execution";
        style=solid; color="#333333"; fontname="sans-serif";
        init [label="init(node) → H"];
        child1 [label="recurse(child₁) → R"];
        acc1 [label="accumulate(&mut H, &R)"];
        child2 [label="recurse(child₂) → R"];
        acc2 [label="accumulate(&mut H, &R)"];
        fin [label="finalize(&H) → R"];

        init -> child1 -> acc1 -> child2 -> acc2 -> fin;
    }
}
```

## Three pieces

The function above takes three things as parameters. hylic
gives each a name and a type:

**Treeish** — the tree structure. Given a node, visit its children:

```rust
{{#include ../../../../hylic/src/graph/edgy.rs:edgy_struct}}
```

`Treeish<N>` is an alias for `Edgy<N, N>` — an edge function where
nodes and edges are the same type:

```rust
{{#include ../../../../hylic/src/graph/edgy.rs:treeish_alias}}
```

You construct one by providing a function from node to children:

```rust
{{#include ../../../src/docs_examples.rs:treeish_constructor}}
```

The callback-based signature (`Fn(&N, &mut dyn FnMut(&N))`) means
zero allocation per visit. The `treeish()` constructor wraps a
`Vec`-returning function into this form.

The node type N can be anything — a nested struct, an integer index
into an adjacency list, a string key into a map, or a reference
resolved through I/O. The tree structure lives in the treeish
function, not in the data.

**Fold** — the computation. In the Shared domain, three closures behind Arc:

```rust
{{#include ../../../../hylic/src/domain/shared/fold.rs:fold_struct}}
```

Other [domains](../design/domains.md) use Rc (Local) or Box (Owned)
— same operations, different boxing. The fold type doesn't carry the
domain; the [executor](../executor-design/exec_pattern.md) does.

- `init`: node → heap (initialize working state)
- `accumulate`: heap × child result → heap (fold in one child)
- `finalize`: heap → result (produce the node's answer)

The intermediate heap `H` lets you accumulate children one at a time
without collecting them first. `simple_fold` is a shorthand where
`H = R` and finalize is clone:

```rust
{{#include ../../../src/docs_examples.rs:simple_fold_example}}
```

**Executor** — the strategy. Controls HOW the recursion runs:

```rust
{{#include ../../../src/docs_examples.rs:exec_usage}}
```

Two built-in executors:

| Executor | Traversal | Domains |
|---|---|---|
| `dom::FUSED` | Callback (sequential) | all |
| [Funnel](../funnel/overview.md) | CPS work-stealing (parallel) | Shared |

Each implements the `Executor<N, R, D>` trait — parameterized by
a boxing [domain](../design/domains.md).
See [Executor architecture](../executor-design/exec_pattern.md) for details.

## The separation

<!-- -->

```dot process
digraph {
    rankdir=LR;
    node [shape=box, style="rounded,filled", fillcolor="#f5f5f5", fontname="monospace", fontsize=11];
    edge [fontname="sans-serif", fontsize=10];

    Treeish [label="Treeish<N>\nstructure"];
    Fold [label="Fold<N, H, R>\ncomputation"];
    Exec [label="dom::FUSED\nstrategy"];
    Domain [label="Domain\n(Shared / Local / Owned)", fillcolor="#fff3cd"];
    R [label="R", shape=ellipse, style=filled, fillcolor="#d4edda"];

    Treeish -> Exec [label="graph"];
    Fold -> Exec [label="algebra"];
    Domain -> Exec [label="boxing", style=dashed];
    Exec -> R [label="run"];
}
```

The fold doesn't know about the tree. The tree doesn't know about
the fold. The executor connects them. The domain determines how
closures are stored — but the fold and treeish don't carry it;
the executor does.

Everything in hylic reduces to `dom::FUSED.run(&fold, &treeish, &root)`.
Even `GraphWithFold::run` (the pipeline for lazy tree discovery)
is just one manual fold step for the entry point, then `executor.run`
for each child tree — see [Entry points](./entry.md).

## Under the hood: operations traits

The executor's recursion engine doesn't know about Arc, Rc, or Box.
It takes `&impl FoldOps<N, H, R>` and `&impl TreeOps<N>` — pure
operation traits:

```rust
{{#include ../../../../hylic/src/ops/fold.rs:foldops_trait}}
```

```rust
{{#include ../../../../hylic/src/ops/tree.rs:treeops_trait}}
```

The standard `Fold<N, H, R>` and `Treeish<N>` implement these traits.
So do `local::Fold`, `owned::Fold`, and any user-defined struct with
the right methods. The executor is generic over these traits — when
called with a concrete struct, the compiler inlines completely.

See [Domain system](../design/domains.md) for how domains connect
operations to storage.

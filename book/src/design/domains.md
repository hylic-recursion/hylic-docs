# Domain system

A **domain** is a boxing strategy — how closures inside Fold and
Treeish are stored. Three built-in domains cover the spectrum from
maximum capability to zero overhead.

## The three domains

<!-- -->

```dot process
digraph {
    rankdir=LR;
    node [shape=box, style="rounded,filled", fontname="monospace", fontsize=11];
    edge [fontname="sans-serif", fontsize=10];

    shared [label="Shared\nArc<dyn Fn + Send + Sync>\nClone, Send, Sync", fillcolor="#d4edda"];
    local [label="Local\nRc<dyn Fn>\nClone, !Send", fillcolor="#fff3cd"];
    owned [label="Owned\nBox<dyn Fn>\n!Clone, !Send", fillcolor="#f8d7da"];

    shared -> local [label="less capability", style=dashed, dir=back];
    local -> owned [label="less capability", style=dashed, dir=back];
}
```

| Domain | Storage | Clone | Send+Sync | Executors | Use case |
|--------|---------|-------|-----------|-----------|----------|
| **Shared** | `Arc<dyn Fn + Send + Sync>` | yes | yes | all | Rayon, Lifts, pipelines |
| **Local** | `Rc<dyn Fn>` | yes | no | Fused, Sequential, Pool, Lifts | lighter refcount |
| **Owned** | `Box<dyn Fn>` | no | no | Fused, Sequential, Pool | zero refcount |

## Domain modules as the single entry point

Each domain has its own module that re-exports everything needed:

```rust
use hylic::domain::shared as dom;   // the standard choice
use hylic::domain::local as dom;    // lighter refcount
use hylic::domain::owned as dom;    // zero overhead
```

Implementation modules (`fold/`, `graph/`, `pipeline.rs`, `parref.rs`)
are `pub(crate)` — internal. Users access types and constructors
exclusively through domain modules. One way in.

## The `Domain` trait

<!-- -->

```rust
{{#include ../../../../hylic/src/domain/mod.rs:domain_trait}}
```

Each domain marker implements this trait, providing concrete Fold
and Treeish types via GATs (Generic Associated Types). The executor
trait is parameterized by the domain:

```rust
{{#include ../../../../hylic/src/cata/exec/mod.rs:executor_trait}}
```

The compiler resolves `D::Fold<H, R>` to the concrete type — e.g.,
`shared::Fold<N, H, R>` when `D = Shared`.

## FoldOps and TreeOps — the universal interface

The operations traits sit above all domains:

```dot process
digraph {
    rankdir=TB;
    node [shape=box, style="rounded,filled", fillcolor="#f5f5f5", fontname="monospace", fontsize=11];
    edge [fontname="sans-serif", fontsize=10];

    foldops [label="FoldOps<N, H, R>\ninit / accumulate / finalize", fillcolor="#e8e8e8"];
    treeops [label="TreeOps<N>\nvisit / apply", fillcolor="#e8e8e8"];

    sf [label="shared::Fold\n(Arc)"];
    lf [label="local::Fold\n(Rc)"];
    of [label="owned::Fold\n(Box)"];
    uf [label="UserStruct\n(no boxing)"];

    st [label="shared::Treeish\n(Arc)"];
    lt [label="local::Treeish\n(Rc)"];
    ot [label="owned::Treeish\n(Box)"];

    foldops -> sf [dir=back];
    foldops -> lf [dir=back];
    foldops -> of [dir=back];
    foldops -> uf [dir=back];

    treeops -> st [dir=back];
    treeops -> lt [dir=back];
    treeops -> ot [dir=back];
}
```

Any type implementing `init`/`accumulate`/`finalize` is a fold. Any
type implementing `visit` is a graph. The executor's recursion engine
takes `&impl FoldOps + &impl TreeOps` — fully generic.

## Why the domain is on the executor, not the fold

Fold and Treeish have no domain parameter: `Fold<N, H, R>` and
`Treeish<N>`. This keeps types simple. The domain marker lives on
the executor: `FusedIn<D>(PhantomData<D>)`.

This solves a type inference problem: if the domain were on the fold,
the compiler couldn't determine D from the argument types when multiple
`Executor` impls exist (GATs are not injective — the compiler can't
reason "this fold type came from Shared, therefore D = Shared"). With
D on the executor, each const has exactly one impl.

## Constructing folds in different domains

Same closures, different constructor:

```rust
{{#include ../../../src/docs_examples.rs:domain_switching}}
```

The closures are domain-independent. The constructor selects the
boxing strategy. To switch domains, change the import — the code
stays the same.

## When to use which domain

**Shared** — the default. Use when:
- You need Rayon (parallel execution)
- You use Lifts (Explainer, ParLazy, ParEager)
- You use GraphWithFold pipelines (they need Clone)
- You need to share folds across threads

**Local** — lighter refcount, still parallel-capable:
- Rc clone is ~1ns vs Arc's ~5ns
- Works with Pool executor (via SyncRef)
- Works with Lifts (ParLazy, ParEager — domain-generic via ConstructFold)

**Owned** — zero refcount:
- Box is the cheapest storage
- The fold can't be cloned, so no Lifts, no pipelines
- Works with Fused, Sequential, and Pool
- Good for benchmarking: shows the framework's raw overhead

Most users should use Shared and never think about domains.

# Executor architecture

The executor controls how the tree recursion is carried out. The fold
defines the computation; the graph defines the tree structure; the
executor decides the traversal order, parallelism strategy, and
resource lifecycle.

Every executor is a `Copy` Spec — a small value that fully describes
a computation strategy. Calling `.run()` on a Spec turns the
description into execution: for Fused this means direct recursion,
for Funnel it means creating a scoped thread pool and running a CPS
work-stealing traversal. Resource creation and cleanup are internal
to the executor.

## The uniform interface

```rust,no_run
use hylic::prelude::*;

FUSED.run(&fold, &graph, &root);                              // sequential
exec(funnel::Spec::default(8)).run(&fold, &graph, &root);     // parallel
```

The same `.run()` method, the same call shape. The fold and graph
are borrowed; the executor manages its own resources.

## Built-in executors

**Fused** — sequential callback-based recursion. Supports all
domains and all graph types (`G: TreeOps<N>`). Uses no resources
beyond the call stack. Equivalent in cost to hand-written recursion.

**Funnel** — parallel CPS work-stealing. Requires `G: Send + Sync`
on the graph type (shared across a scoped thread pool). Configurable
through three compile-time policy axes: queue topology, accumulation
strategy, and wake policy. See [Funnel](../funnel/overview.md).

## Domain and graph requirements

The `Executor` trait is parameterized by four type parameters:
`N` (node), `R` (result), `D` (domain), and `G` (graph). The domain
controls the fold type (`D::Fold<H, R>`). The graph type `G` is
constrained per executor implementation — Fused accepts any
`TreeOps<N>`, Funnel requires `Send + Sync`. This means the fold
domain and the graph type are independent choices.

See [Domain integration](../executor-design/domain_integration.md)
for the type-level details and
[The Exec pattern](../executor-design/exec_pattern.md) for the
Spec/Session/Exec lifecycle.

# Executor architecture

The executor controls **how** the tree recursion runs. The fold says
what to compute; the treeish says where the children are; the executor
decides the traversal order and parallelism strategy.

Every executor is a `Copy` Spec — data that describes a strategy.
Calling `.run()` refunctionalizes it: turns data into computation.
Resource management (thread pools, arenas) is internal.

## The uniform API

```rust
use hylic::domain::shared as dom;

dom::FUSED.run(&fold, &graph, &root);                              // sequential
dom::exec(funnel::Spec::default(8)).run(&fold, &graph, &root);     // parallel
```

Same method. Same shape. Resource-needing executors create and
destroy their resources inside `.run()`. Zero-resource executors
run directly. The user doesn't know or care.

## The two built-in executors

**Fused** — sequential, all domains. Callback-based recursion, zero
allocation. `Resource = ()`, `Session = Self`.

**Funnel** — parallel, Shared domain. CPS work-stealing with three
policy axes. `Resource = &Pool`, `Session = Session<P>`.
See [Funnel](../funnel/overview.md).

## Domain support

Fused supports all domains (borrows, never clones). Funnel requires
`N: Clone + Send, R: Clone + Send`. See
[Domain integration](../executor-design/domain_integration.md).

## Deep dives

- [The Exec pattern](../executor-design/exec_pattern.md) — `Exec<D, S>`,
  `ExecutorSpec`, the three usage tiers, defunctionalization
- [Domain integration](../executor-design/domain_integration.md) —
  `Domain<N>` GATs, forward resolution
- [Policy traits](../executor-design/policy_traits.md) —
  zero-cost configuration via GATs
- [Funnel executor](../funnel/overview.md) — the parallel executor

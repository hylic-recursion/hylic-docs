# Funnel: Parallel Fused Hylomorphism

The funnel executor parallelizes a fused hylomorphism — an unfold
(tree traversal) composed with a fold (bottom-up accumulation) where
the intermediate tree is never materialized. Children are discovered
one at a time through a push-based callback, processed concurrently
across worker threads, and their results flow back to the parent
through defunctionalized continuations.

## What a fused hylomorphism is

A **hylomorphism** composes an unfold (anamorphism) with a fold
(catamorphism). The unfold generates a tree structure from a seed;
the fold consumes it bottom-up. When fused, the two interleave:
each node is produced, its children recursively processed, and their
results accumulated — without materializing the tree.

In hylic terms: a `Treeish<N>` (the coalgebra) exposes
`visit(&node, |child| ...)` and a `Fold<N, H, R>` (the algebra,
[factored as init/accumulate/finalize](../design/milewski.md))
provides the per-node bracket. The executor calls `visit` to
discover children, recursively processes each, accumulates their `R`
results into the parent's `H` heap, and finalizes. The intermediate
tree is never materialized as a data structure.

The funnel parallelizes this: children beyond the first are pushed
to a work-stealing queue. Worker threads steal and process subtrees
concurrently. Results flow back through continuations to the parent's
accumulator. The challenge is coordinating the fold — detecting when
all children are done, accumulating their results, and cascading
upward — without locks, without allocation on the critical path.

## Design values

Four properties define the funnel's design:

```dot process
digraph {
  rankdir=LR;
  node [shape=box, fontname="sans-serif", fontsize=10, style="rounded,filled"];
  edge [fontname="sans-serif", fontsize=9, style=dashed];

  iter [label="Iterator-based\ntraversal\n(push callback)", fillcolor="#d4edda"];
  par [label="Fully\nparallel\n(work-stealing)", fillcolor="#cce5ff"];
  fused [label="Fused\nunfold+fold\n(no tree built)", fillcolor="#fff3cd"];
  zero [label="Zero alloc\nhot path\n(arenas + inline)", fillcolor="#f8d7da"];

  iter -> par -> fused -> zero;
}
```

1. **Iterator-based traversal.** The graph exposes
   `visit(&node, |child| ...)`. Children arrive one at a time.
   There is no `children(&node) -> Vec<N>`.

2. **Fully parallel.** Each child beyond the first is pushed to a
   work-stealing queue. Workers steal and process subtrees concurrently.
   The first child is walked inline — zero queue overhead on the DFS spine.

3. **Fused unfold+fold.** Results accumulate into the parent as they
   arrive (streaming) or in bulk by the last thread (finalize). The
   tree is never materialized.

4. **Zero allocation on the hot path.** Tasks are enum variants stored
   inline in deque slots. Multi-child accumulators are arena-allocated.
   Single-child nodes carry their heap inside the continuation. No
   `Box<dyn FnOnce>`, no `Arc` per task.

## Where funnel sits

| Executor | Parallelism | Unfold/fold fusion | Task repr | Allocation |
|---|---|---|---|---|
| **Fused** | none | fully fused | stack frames | zero |
| **Funnel** | CPS + work-stealing | fully fused | `FunnelTask` enum | arenas |

Fused is the sequential baseline — zero overhead, callback-based
recursion on a single thread. Funnel preserves the fused property
while adding parallelism through CPS (continuation-passing style)
and work-stealing queues. The fold/graph are unchanged between the
two — only the executor differs.

Both use the same [`Exec<D, S>`](../executor-design/exec_pattern.md)
type-level pattern. Funnel's policy system is an instance of the
generic [Spec → Store → Handle](../executor-design/policy_traits.md)
pattern for zero-cost executor configuration.

## Module map

The funnel's code is organized into four clusters:

```dot process
digraph {
  rankdir=LR;
  node [shape=box, fontname="monospace", fontsize=9, style="rounded,filled"];
  edge [fontname="monospace", fontsize=8];

  subgraph cluster_cps {
    label="cps/"; style=dashed; fillcolor="#f0f0ff"; color="#888888";
    walk [label="walk.rs\nwalk_cps\nfire_cont", fillcolor="#cce5ff"];
    cont [label="cont.rs\nFunnelTask\nCont, ChainNode", fillcolor="#cce5ff"];
    chain [label="chain.rs\nFoldChain\nticket system", fillcolor="#cce5ff"];
  }

  subgraph cluster_dispatch {
    label="dispatch/"; style=dashed; fillcolor="#fff8f0"; color="#888888";
    run [label="mod.rs\nrun_fold", fillcolor="#fff3cd"];
    worker [label="worker.rs\nWorkerCtx\nworker_loop", fillcolor="#fff3cd"];
    view [label="view.rs\nFoldView", fillcolor="#fff3cd"];
  }

  subgraph cluster_policy {
    label="policy/"; style=dashed; fillcolor="#f0fff0"; color="#888888";
    pmod [label="mod.rs\nFunnelPolicy\nPolicy<Q,A,W>", fillcolor="#d4edda"];
    subgraph cluster_queue {
      label="queue/"; style=dotted;
      qmod [label="mod.rs\nWorkStealing\nTaskOps", fillcolor="#e8f5e8"];
      pw [label="per_worker.rs", fillcolor="#e8f5e8"];
      sh [label="shared.rs", fillcolor="#e8f5e8"];
    }
    subgraph cluster_acc {
      label="accumulate/"; style=dotted;
      amod [label="mod.rs\nAccumulateStrategy", fillcolor="#e8f5e8"];
    }
    subgraph cluster_wake {
      label="wake/"; style=dotted;
      wmod [label="mod.rs\nWakeStrategy", fillcolor="#e8f5e8"];
    }
  }

  subgraph cluster_infra {
    label="infra/"; style=dashed; fillcolor="#fff0f0"; color="#888888";
    slab [label="segmented_slab.rs\nSegmentedSlab<T>", fillcolor="#f8d7da"];
    arena [label="arena.rs\nArena<T>\n(wraps slab)", fillcolor="#f8d7da"];
    cont_arena [label="cont_arena.rs\nContArena<T>\n(wraps slab)", fillcolor="#f8d7da"];
    deque [label="deque.rs\nWorkerDeque<T>", fillcolor="#f8d7da"];
    ec [label="eventcount.rs\nEventCount", fillcolor="#f8d7da"];
    slab -> arena [style=dotted, dir=back];
    slab -> cont_arena [style=dotted, dir=back];
  }

  pool [label="pool.rs\nPool, Job\nPoolState\ndispatch()", fillcolor="#e8e8e8"];
  modrs [label="mod.rs\nSpec<P>\nSession<P>", fillcolor="#e8e8e8"];

  walk -> cont; walk -> chain; walk -> worker;
  cont -> chain; cont -> arena; cont -> cont_arena;
  run -> walk; run -> worker; run -> pool;
  worker -> walk; worker -> qmod;
  qmod -> pw [style=dashed]; qmod -> sh [style=dashed];
  pw -> deque;
  pool -> ec;
  pmod -> qmod; pmod -> amod; pmod -> wmod;
  modrs -> run; modrs -> pool; modrs -> pmod;
}
```

## Three behavioral axes

The funnel is parameterized along three independent axes, all
resolved at compile time through the `FunnelPolicy` trait:

```rust
{{#include ../../../../hylic/src/cata/exec/variant/funnel/policy/mod.rs:funnel_policy_trait}}
```

Each axis is a trait with its own `Spec`, `Store`/`State`, and
implementations. The `Policy<Q, A, W>` struct bundles any combination:

```rust
{{#include ../../../../hylic/src/cata/exec/variant/funnel/policy/mod.rs:policy_struct}}
```

Named presets are type aliases:

```rust
{{#include ../../../../hylic/src/cata/exec/variant/funnel/policy/mod.rs:named_presets}}
```

```dot process
digraph {
  rankdir=TB;
  node [shape=box, fontname="sans-serif", fontsize=10, style="rounded,filled"];
  edge [fontname="sans-serif", fontsize=9];

  policy [label="FunnelPolicy\n(one type parameter)", fillcolor="#e2d9f3"];

  q [label="Queue\nWorkStealing", fillcolor="#cce5ff"];
  a [label="Accumulate\nAccumulateStrategy", fillcolor="#fff3cd"];
  w [label="Wake\nWakeStrategy", fillcolor="#d4edda"];

  pw [label="PerWorker\nChase-Lev + bitmask", fillcolor="#cce5ff", fontsize=9];
  shared [label="Shared\nStealQueue", fillcolor="#cce5ff", fontsize=9];
  arrive [label="OnArrival\nstreaming sweep", fillcolor="#fff3cd", fontsize=9];
  finalize [label="OnFinalize\nbulk sweep", fillcolor="#fff3cd", fontsize=9];
  every [label="EveryPush", fillcolor="#d4edda", fontsize=9];
  once [label="OncePerBatch", fillcolor="#d4edda", fontsize=9];
  everyk [label="EveryK<K>", fillcolor="#d4edda", fontsize=9];

  policy -> q; policy -> a; policy -> w;
  q -> pw; q -> shared;
  a -> arrive; a -> finalize;
  w -> every; w -> once; w -> everyk;
}
```

See [Policies](policies.md) for the full decision guide and
benchmark-informed recommendations.

## Reading order

| Page | What you learn |
|---|---|
| [CPS walk](cps_walk.md) | The downward pass: how nodes are processed and tasks created |
| [Continuations](continuations.md) | `FunnelTask`, `Cont`, `ChainNode`, `RootCell` — the CPS data types |
| [Cascade](cascade.md) | `fire_cont`: the trampolined upward pass |
| [Ticket system](ticket_system.md) | Packed `AtomicU64` for exactly-one-finalizer detection |
| [Pool and dispatch](pool_dispatch.md) | Thread pool, `Job` struct, the `dispatch()` CPS lifecycle |
| [Queue strategies](queue_strategies.md) | PerWorker (Chase-Lev + bitmask) vs Shared (StealQueue) |
| [Accumulation](accumulation.md) | OnArrival (streaming sweep) vs OnFinalize (bulk) |
| [Policies](policies.md) | `FunnelPolicy` GAT, three axes, named presets, decision guide |
| [Infrastructure](infrastructure.md) | Arena, ContArena, WorkerDeque, EventCount |
| [Testing](testing.md) | Correctness, stress, interleaving proof |

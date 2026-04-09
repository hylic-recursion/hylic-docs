# CPS Walk: The Downward Pass

`walk_cps` is the core of the funnel executor. It processes one node
at a time: initializes the fold heap, iterates children through the
graph's push-based visitor, and branches on the child count. It is a
**void function** — results flow through continuations, not return
values. This is what makes cross-thread result delivery possible
without blocking.

## The algorithm

```rust
{{#include ../../../../hylic/src/cata/exec/variant/funnel/cps/walk.rs:walk_cps}}
```

The function takes `(wctx, node, cont)`:
- `wctx`: per-worker context (queue handle + wake state)
- `node`: the graph node to process
- `cont`: what to do with this node's result

It loops (trampolined for the inline child case), processing one
node per iteration.

## Child-count branching

After `graph.visit` returns, the child count determines the control flow:

```dot process
digraph {
  rankdir=TB;
  node [shape=box, style="rounded,filled", fontname="sans-serif", fontsize=10];
  edge [fontname="sans-serif", fontsize=9];

  walk [label="walk_cps(node, cont)", fillcolor="#fff3cd"];
  init [label="fold.init(&node)\ngraph.visit(callback)", fillcolor="#f5f5f5"];
  leaf [label="0 children (leaf)\nfinalize → fire_cont(cont, result)", fillcolor="#d4edda"];
  single [label="1 child\nCont::Direct { heap }\nloop continues with child", fillcolor="#cce5ff"];
  multi [label="2+ children\nChainNode + FoldChain\nset_total\nloop continues with child₀", fillcolor="#f8d7da"];
  push [label="children 1..K:\npush_task(Walk{child, Slot{i}})", fillcolor="#f5f5f5", style="rounded,dashed"];

  walk -> init;
  init -> leaf [label="0"];
  init -> single [label="1"];
  init -> multi [label="≥2"];
  init -> push [style=dashed, label="during visit"];
}
```

**Leaf (0 children):** Finalize the heap and call
[`fire_cont`](cascade.md) with the original continuation. This is
the base case — the upward cascade begins here.

**Single child (1):** No `ChainNode` needed. The heap moves into a
`Cont::Direct`, the parent continuation is stored in the `ContArena`,
and the loop continues with the child. Zero queue interaction, zero
atomic operations.

**Multi-child (2+):** A `ChainNode` is allocated in the arena
(lazily, on child 2 — not child 1). Children 1..K are pushed as
`FunnelTask::Walk` to the queue. Then `set_total` records the child
count in the [ticket system](ticket_system.md). The loop continues
with child 0 (inline walk).

## First-child inlining

Child 0 is ALWAYS walked inline — a continuation of the current
thread's DFS spine, with zero queue overhead. Siblings are pushed
to the queue for workers to steal. This gives every active thread
a guaranteed DFS path from its entry point to a leaf:

```dot process
digraph {
  rankdir=TB;
  node [fontname="monospace", fontsize=10, style="filled"];
  root [label="root\nThread 0", fillcolor="#ffcccc"];
  c0 [label="c0\nThread 0 (inline)", fillcolor="#ffcccc"];
  c1 [label="c1\nThread 1 (stolen)", fillcolor="#ccccff"];
  c2 [label="c2\nThread 2 (stolen)", fillcolor="#ccffcc"];
  c00 [label="c00\nThread 0 (inline)", fillcolor="#ffcccc"];
  c01 [label="c01\nstolen", fillcolor="#ffffcc"];
  root -> c0 [color=red, penwidth=2, label="inline"];
  root -> c1 [style=dashed]; root -> c2 [style=dashed];
  c0 -> c00 [color=red, penwidth=2, label="inline"];
  c0 -> c01 [style=dashed];
}
```

Red edges = inline walks (zero queue cost). Dashed = queue
submissions. Thread 0 walks root → c0 → c00 → ... → leaf without
touching the queue at any level. This is structurally equivalent to
Cilk's continuation-stealing, inverted: we push sibling tasks (child
stealing) instead of stealing the parent's continuation.

Three compounding effects make this critical:

- **Zero-queue spine.** For depth D, one thread processes D nodes
  with no push/pop overhead (~20-50ns saved per level).
- **Cache warmth.** `ChainNode`s allocated on the way down are in
  L1 cache on the way up via [`fire_cont`](cascade.md).
- **Reduced contention.** One fewer task per level competing for
  deque access.

## Defunctionalization

Tasks are data, not closures:

```rust
{{#include ../../../../hylic/src/cata/exec/variant/funnel/cps/cont.rs:funnel_task}}
```

`FunnelTask::Walk` pairs a child node with its continuation — plain
data stored inline in deque slots. No `Box<dyn FnOnce>`, no closure
capture, no vtable. The `execute_task` function is the apply:

```rust
{{#include ../../../../hylic/src/cata/exec/variant/funnel/cps/walk.rs:execute_task}}
```

This is the Reynolds/Danvy defunctionalization transformation applied
to parallel work items.

## Streaming submission

Children are pushed to the queue **during** `graph.visit`, not after.
Workers can steal siblings while the parent is still discovering
more children. `append_slot` is called per child inside the callback;
`set_total` is called after `graph.visit` returns. Between these two
events, workers may deliver results to already-appended slots. The
[ticket system](ticket_system.md) handles this race.

## Task submission and wake

```rust
{{#include ../../../../hylic/src/cata/exec/variant/funnel/dispatch/worker.rs:push_task}}
```

`push` goes through the policy's queue handle. If the queue is
full, the task is executed inline (Cilk overflow protocol). Otherwise,
the wake strategy decides whether to notify a parked worker.

## Worked example

A sum fold over tree `R(A(D,E), B, C)` where D, E, B, C are leaves.
Thread 0 is the caller; threads 1-2 are workers.

```dot process
digraph {
  rankdir=TB;
  node [shape=box, style="rounded,filled", fontname="monospace", fontsize=8];
  edge [fontname="sans-serif", fontsize=7];

  subgraph cluster_t0 {
    label="Thread 0 (caller)"; style=filled; fillcolor="#ffcccc22"; color="#cc0000";
    fontname="sans-serif"; fontsize=9;
    t0_1 [label="walk_cps(R, Root)\ninit heap_R\nvisit: A=child₀\nB → push Walk{B, Slot{R,1}}\nC → push Walk{C, Slot{R,2}}\nset_total(3)", fillcolor="#ffcccc"];
    t0_2 [label="walk_cps(A, Slot{R,0})\n← inline\nD=child₀, E → push\nset_total(2)", fillcolor="#ffcccc"];
    t0_3 [label="walk_cps(D, Slot{A,0})\nleaf → fire_cont", fillcolor="#ffcccc"];
    t0_4 [label="fire_cont(Slot{A,0})\ndeliver → not last\nreturn to help loop", fillcolor="#ffdddd"];
    t0_1 -> t0_2 -> t0_3 -> t0_4;
  }

  subgraph cluster_t1 {
    label="Thread 1"; style=filled; fillcolor="#ccccff22"; color="#0000cc";
    fontname="sans-serif"; fontsize=9;
    t1_1 [label="steal Walk{B, Slot{R,1}}\nleaf → fire_cont\ndeliver → not last", fillcolor="#ccccff"];
    t1_2 [label="steal Walk{E, Slot{A,1}}\nleaf → fire_cont\nLAST for A → sweep\n→ fire_cont(Slot{R,0})\nnot last for R", fillcolor="#ddddff"];
    t1_1 -> t1_2;
  }

  subgraph cluster_t2 {
    label="Thread 2"; style=filled; fillcolor="#ccffcc22"; color="#00cc00";
    fontname="sans-serif"; fontsize=9;
    t2_1 [label="steal Walk{C, Slot{R,2}}\nleaf → fire_cont\nLAST for R → sweep\n→ fire_cont(Root)\nfold_done = true", fillcolor="#ccffcc"];
  }

  t0_1 -> t1_1 [style=dashed, label="push B", color="#888888"];
  t0_2 -> t1_2 [style=dashed, label="push E", color="#888888"];
  t0_1 -> t2_1 [style=dashed, label="push C", color="#888888"];
}
```

- Thread 0 walks the left spine (R→A→D) inline
- Thread 1 steals B, then E — becomes finalizer for A, cascades
  A's result to R
- Thread 2 steals C — becomes finalizer for R, fires `Cont::Root`
- The fold completes when any thread fires `Cont::Root`

## Cross-references

- [Continuations](continuations.md) — `Cont`, `FunnelTask`, `ChainNode`
- [Cascade](cascade.md) — `fire_cont`: the trampolined upward pass
- [Ticket system](ticket_system.md) — how `set_total` determines the
  finalizer
- [Queue strategies](queue_strategies.md) — how `push_task` dispatches
  to PerWorker or Shared

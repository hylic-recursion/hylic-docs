# Continuations: CPS Data Types

Three types carry the fold's state through the CPS pipeline:
`FunnelTask` (the parallelism boundary), `Cont` (the continuation),
and `ChainNode` (the multi-child accumulator). A fourth, `RootCell`,
is the terminal sink for the final result. Together they replace
implicit stack frames with explicit data that can be created on one
thread and consumed on another.

## FunnelTask

```rust
{{#include ../../../../hylic/src/exec/variant/funnel/cps/cont.rs:funnel_task}}
```

The unit of parallelism. Stored inline in deque slots (PerWorker)
or queue segments (Shared). No heap allocation per task — the enum
variant IS the data. `N` must be `Clone + Send` (cloned during
`graph.visit`, sent across threads). `R` must be `Send` (results are
moved across threads via destructive slot reads). `H` has no bounds
— it travels inside `Cont::Direct`.

## Cont

```rust
{{#include ../../../../hylic/src/exec/variant/funnel/cps/cont.rs:cont_enum}}
```

The defunctionalized continuation. Tells [`fire_cont`](cascade.md)
what to do with a result:

### `Cont::Root`

Terminal. Created once per fold. When `fire_cont` receives it, the
fold is complete: the result is written to the `RootCell` and
`fold_done` is signaled. Size: 8 bytes (one raw pointer).

The `RootCell` lives on `run_fold`'s stack — no heap allocation.
The raw pointer is safe because the scoped pool guarantees all
workers complete before `run_fold` returns.

### `Cont::Direct`

Single-child fast path. The heap value travels WITH the continuation
— no `ChainNode`, no `FoldChain`, no atomics. `parent_idx` is a
`ContIdx(u32)` into the `ContArena`. When `fire_cont` receives it:
accumulate the result into the heap, finalize, take the parent
continuation from the arena, continue the loop.

Size: `sizeof(H) + 4` bytes.

### `Cont::Slot`

Multi-child delivery. Two `u32` indices: `node` (arena index to the
`ChainNode`) and `slot` (which position in the `FoldChain`). When
`fire_cont` receives it: deliver the result to the slot, check the
[ticket](ticket_system.md). If this was the last event, sweep/finalize
the chain and take the parent continuation. If not, return — another
thread will finalize.

Size: 8 bytes (two `u32`, regardless of `H` or `R`).

## ChainNode

```rust
{{#include ../../../../hylic/src/exec/variant/funnel/cps/cont.rs:chain_node}}
```

Arena-allocated. Created lazily on child 2 (never for single-child
nodes). Contains:
- `chain`: the `FoldChain` — slot cells, heap, ticket state
- `parent_cont`: the continuation of the creating node, moved out
  exactly once by the finalizing thread via `take_parent_cont()`

## Continuation graph

For a tree with root R, child A (2 children: C, D), and child B
(1 child: E, leaf):

```dot process
digraph {
  rankdir=BT;
  node [fontname="monospace", fontsize=10, style="rounded,filled"];
  root [label="Cont::Root\n*const RootCell\n(stack-local)", fillcolor="#ffcccc", shape=doubleoctagon];
  chainR [label="ChainNode(R)\nFoldChain\nparent→Root", fillcolor="#cce5ff"];
  slotR0 [label="Slot{R, 0}", fillcolor="#d4edda"];
  slotR1 [label="Slot{R, 1}", fillcolor="#d4edda"];
  directB [label="Direct{heap_B}\nparent→Slot{R,1}", fillcolor="#fff3cd"];
  leafC [label="leaf C\nfinalize → fire_cont", fillcolor="#f5f5f5"];
  leafD [label="leaf D\nfinalize → fire_cont", fillcolor="#f5f5f5"];
  leafE [label="leaf E\nfinalize → fire_cont", fillcolor="#f5f5f5"];

  slotR0 -> chainR [label="deliver"];
  slotR1 -> chainR [label="deliver"];
  chainR -> root [label="cascade"];
  directB -> slotR1 [label="acc+fin\ncascade"];
  leafC -> slotR0;
  leafD -> slotR0 [style=invis];
  leafE -> directB;
}
```

Leaf C finalizes, delivers to Slot{R,0}. Leaf E finalizes, fires
Direct for B (accumulates + finalizes), delivers to Slot{R,1}.
Whichever delivery is last (ticket) sweeps ChainNode(R) and fires
Root.

## Data ownership

Each CPS type lives in a specific memory region:

```dot process
digraph {
  rankdir=LR;
  node [shape=box, fontname="monospace", fontsize=9, style="rounded,filled"];
  edge [fontname="sans-serif", fontsize=8];

  subgraph cluster_stack {
    label="run_fold stack"; style=filled; fillcolor="#ffcccc22"; color="#cc0000";
    fontname="sans-serif"; fontsize=9;
    wctx [label="WalkCtx\nrefs to fold, graph\narenas, view", fillcolor="#ffcccc"];
    job [label="Job\nfn ptr + data ptr", fillcolor="#ffcccc"];
  }

  subgraph cluster_arena {
    label="Arenas (bump alloc)"; style=filled; fillcolor="#d4edda22"; color="#28a745";
    fontname="sans-serif"; fontsize=9;
    cn [label="ChainNode\nFoldChain + parent_cont", fillcolor="#d4edda"];
    ca [label="ContArena\nparent Conts\n(for bf=1 chains)", fillcolor="#d4edda"];
  }

  subgraph cluster_deque {
    label="Deque / StealQueue"; style=filled; fillcolor="#cce5ff22"; color="#004085";
    fontname="sans-serif"; fontsize=9;
    task [label="FunnelTask::Walk\nchild + Cont\n(inline in slot)", fillcolor="#cce5ff"];
  }

  subgraph cluster_stack2 {
    label="run_fold stack (local)"; style=filled; fillcolor="#fff3cd22"; color="#cca000";
    fontname="sans-serif"; fontsize=9;
    root [label="RootCell\nresult + done flag\n(stack-allocated)", fillcolor="#fff3cd"];
  }

  task -> cn [label="Slot: ArenaIdx", style=dashed];
  task -> ca [label="Direct: ContIdx", style=dashed];
  task -> root [label="Root: raw ptr", style=dashed];
}
```

Deque stores tasks inline. Arena indices are `u32` (Copy, no
refcount). The CPS pipeline has **zero heap allocations** on the
critical path — RootCell is stack-local, arenas grow lazily via
[segmented allocation](infrastructure.md), and tasks are stored
inline in deque slots.

## Size summary

| Type | Size | Notes |
|---|---|---|
| `Cont::Root` | 8 bytes | raw pointer to stack-local RootCell |
| `Cont::Direct` | `sizeof(H) + 4` | heap value + `ContIdx(u32)` |
| `Cont::Slot` | 8 bytes | `ArenaIdx(u32) + SlotRef(u32)` |
| `FunnelTask::Walk` | `sizeof(N) + sizeof(Cont) + tag` | stored inline in deque |
| `ChainNode` | `sizeof(FoldChain) + sizeof(Option<Cont>)` | arena-allocated |
| `RootCell` | `sizeof(Option<R>) + 1` | stack-local in `run_fold` |

# Zero-cost performance

The closure-based API (`Fold` from a domain module, plus `Treeish`)
is the ergonomic default. For performance-critical paths, the
graph side admits user-defined `TreeOps` implementations whose
`visit` method monomorphises directly. The fold side does not — the
executor signature pins the fold type to `D::Fold<H, R>` (the
closure-based wrapper). The two `ops` traits are nevertheless the
right vocabulary for thinking about per-node cost.

## The overhead budget

Per node with K children, the fused executor makes these calls
through the closure-based API:

| Call site                       | Count   | Dispatch                       |
|---------------------------------|---------|--------------------------------|
| `fold.init(node)`               | 1       | `dyn Fn` via Arc/Rc/Box        |
| `graph.visit(node, cb)`         | 1       | `dyn Fn` via Arc/Rc/Box        |
| `cb(child)` inside visit        | K       | `&mut dyn FnMut` callback      |
| `fold.accumulate(heap, &r)`     | K       | `dyn Fn` via Arc/Rc/Box        |
| `fold.finalize(heap)`           | 1       | `dyn Fn` via Arc/Rc/Box        |
| **Total**                       | **3+2K**|                                |

Measured: ~0.47 ns per indirect call (well-predicted by the
branch predictor). On a noop workload (bf=8, 200 nodes): ~1.8 µs
above hand-written recursion. On any real workload (>10 µs/node)
the overhead drops below the noise floor.

## Eliminating graph dispatch: implement TreeOps

The executor's graph parameter is generic — `G: TreeOps<N>` —
so any concrete impl is monomorphised at the call site:

```rust
{{#include ../../../src/docs_examples.rs:zero_cost_treeops}}
```

`AdjGraph::visit` is a direct, inlinable loop. Only the callback
`cb: &mut dyn FnMut` is still indirect — K calls per node. The
closure-based fold is still in the picture because executors take
`&D::Fold<H, R>`; replacing it requires a custom executor (below).

## The shape of the trait

`FoldOps` and `TreeOps` are the operation traits any user code
can target:

```rust,ignore
pub trait FoldOps<N, H, R> {
    fn init(&self, node: &N) -> H;
    fn accumulate(&self, heap: &mut H, result: &R);
    fn finalize(&self, heap: &H) -> R;
}

pub trait TreeOps<N> {
    fn visit(&self, node: &N, cb: &mut dyn FnMut(&N));
}
```

The closure-based domain folds (`shared::Fold`, `local::Fold`,
`owned::Fold`) implement `FoldOps` by delegating to their stored
closures. A user-defined `FoldOps` struct is callable from any
custom executor that drives the recursion through the trait —
bypassing the closure layer entirely. The shipped `Fused`
executor's inner loop is exactly this:

```rust
{{#include ../../../../hylic/src/exec/variant/fused/mod.rs:run_inner}}
```

## When the budget matters

| Path                                    | Per-node overhead   | When to use |
|-----------------------------------------|---------------------|-------------|
| Closure-based Fold + Treeish            | 3+2K indirect calls | Default — combinators, lifts, sugars |
| Closure-based Fold + custom TreeOps     | K+1 indirect calls  | Adjacency lists or graph types where the visit path is the hot side |
| Custom executor over `FoldOps + TreeOps`| K indirect calls    | Maximum control; sacrifices the lift / pipeline machinery for one specific shape |

## Why LTO doesn't help

LLVM cannot devirtualise Rust `dyn Fn` calls. Rust does not emit
the `!vcall_visibility` metadata that LLVM's whole-program
devirtualisation would need. Neither thin LTO nor fat LTO changes
this. The trait-based path is the only reliable way to eliminate
dispatch.

See [Benchmarks](./benchmarks.md) for the measured comparison
across all execution modes.

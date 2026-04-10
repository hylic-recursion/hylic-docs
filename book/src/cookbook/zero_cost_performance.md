# Zero-cost performance

The closure-based API (`dom::Fold`, `graph::Treeish`) is the ergonomic
default. For performance-critical paths, you can eliminate all
framework overhead by implementing the operations traits directly.

## The overhead budget

Per node with K children, the fused executor makes these calls
through the closure-based API:

| Call site | Count | Dispatch |
|---|---|---|
| `fold.init(node)` | 1 | `dyn Fn` via Arc/Rc/Box |
| `graph.visit(node, cb)` | 1 | `dyn Fn` via Arc/Rc/Box |
| `cb(child)` inside visit | K | `&mut dyn FnMut` callback |
| `fold.accumulate(heap, &r)` | K | `dyn Fn` via Arc/Rc/Box |
| `fold.finalize(heap)` | 1 | `dyn Fn` via Arc/Rc/Box |
| **Total** | **3 + 2K** | |

Measured: ~0.47ns per indirect call (branch predictor handles well).
On noop (bf=8, 200 nodes): ~1.8us above hand-written recursion.
On any real workload (>10us/node): overhead drops below noise.

## Eliminating fold dispatch: implement FoldOps

Instead of closures, implement `FoldOps` on a concrete struct:

```rust
use hylic::ops::FoldOps;

struct SumFold;

impl FoldOps<TreeNode, u64, u64> for SumFold {
    fn init(&self, node: &TreeNode) -> u64 { node.value }
    fn accumulate(&self, heap: &mut u64, result: &u64) { *heap += result; }
    fn finalize(&self, heap: &u64) -> u64 { *heap }
}
```

The executor takes `&impl FoldOps<N, H, R>`. With a concrete type,
the compiler monomorphizes the recursion engine. All fold method calls
become direct, inlineable function calls. Zero `dyn Fn` overhead.

Works with ALL executors: `dom::FUSED.run(&SumFold, &graph, &root)`.

## Eliminating graph dispatch: implement TreeOps

```rust
use hylic::ops::TreeOps;

struct AdjGraph<'a> {
    adj: &'a [Vec<usize>],
    nodes: &'a [TreeNode],
}

impl<'a> TreeOps<TreeNode> for AdjGraph<'a> {
    fn visit(&self, node: &TreeNode, cb: &mut dyn FnMut(&TreeNode)) {
        for &child_id in &self.adj[node.id] {
            cb(&self.nodes[child_id]);
        }
    }
}
```

`TreeOps::visit` is monomorphized for `AdjGraph`. The visit body is a
direct, inlineable loop. **But** the callback `cb: &mut dyn FnMut` is
still dynamic dispatch — K indirect calls per node.

## The tradeoff

| Path | Ergonomics | Per-node overhead | When to use |
|---|---|---|---|
| Closure-based (`dom::Fold` + `graph::Treeish`) | Best — closures, combinators, map/zip | 3+2K indirect calls | Always, unless profiling shows overhead |
| `FoldOps` struct | Good — one impl block | K+1 indirect calls (visit cb only) | Hot inner loops, millions of nodes |
| `FoldOps` + `TreeOps` | Manual — two impl blocks | K indirect calls (visit cb) | Maximum control, known hot path |

The closure-based API and the trait-based API compose freely. A user
can mix: closure-based Fold with trait-based TreeOps, or vice versa.
The executor doesn't care — it takes `&impl FoldOps` and
`&impl TreeOps`.

## Why LTO doesn't help

LLVM cannot devirtualize Rust `dyn Fn` calls. Rust does not emit the
`!vcall_visibility` metadata that LLVM's whole-program devirtualization
needs. Neither thin LTO nor fat LTO changes this. The `FoldOps` /
`TreeOps` approach is the only reliable way to eliminate dispatch.

See [Benchmarks](./benchmarks.md) for the full performance comparison
across all execution modes.

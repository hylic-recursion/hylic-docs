# Testing

The funnel test suite covers three dimensions: correctness, stress,
and the hylomorphism property. All tests run for both queue strategies
(PerWorker and Shared) via policy-generic test helpers.

## Correctness

Verify that funnel produces the same result as the sequential `Fused`
executor across all named policy presets:

- Default, SharedDefault, WideLight, LowOverhead, PerWorkerArrival
- Tree sizes: 60 nodes (bf=4), 200 nodes (bf=6, bf=20)
- Zero workers (all work done by the caller thread)
- Adjacency-list trees (callback-based `treeish_visit`)
- Wide-tree stress (500 iterations, pool reused)

## Stress

High iteration counts to catch timing-sensitive races:

- **1500 runs** per policy on a reused pool
- **Pool lifecycle**: 5000 create/destroy cycles
- **Mixed policy**: 50k iterations switching between PerWorker and
  Shared on the same pool (mimics criterion benchmark pattern)
- **100k noop iterations**: Shared + OnFinalize and Shared + OnArrival
  at criterion warmup intensity
- **Interleaved policies**: 12.5k iterations alternating four
  policies on one pool

These tests exercise the `dispatch` → `in_job` latch protocol
under the exact conditions that previously triggered SIGSEGV
(high-iteration noop folds with rapid pool reuse).

## Interleaving proof

The hylomorphism property: fold interleaves with traversal across
subtrees. While one subtree is being visited (walk down), another
subtree's results are being accumulated (cascade up).

The test uses a lock-free `TraceLog` (65k-entry ring buffer, atomic
sequence counter) to record visit and accumulate operations with
thread IDs and subtree tags. After 20 attempts on an 85-node tree,
the test asserts that cross-subtree interleaving occurred — proving
that the fold is not merely parallel but genuinely fused.

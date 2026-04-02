<!-- TODO: Write this page based on KB/.plans/continued-perf-apr02/power-user-zero-cost.md
     
     This page should cover:
     1. The closure-based API overhead (dyn Fn per fold/graph operation)
     2. Implementing FoldOps directly on a user struct (eliminates fold dispatch)
     3. Implementing TreeOps directly on a user struct (eliminates graph dispatch)
     4. The visit_inline method (eliminates callback dispatch — pending TreeOps change)
     5. Benchmarks showing the closure path vs trait path vs hand-written
     6. When to use each path (ergonomics vs performance tradeoff)
     
     Source material: KB/.plans/continued-perf-apr02/power-user-zero-cost.md
     See also: KB/.plans/continued-perf-apr02/hard-to-fix/visit-callback-dispatch.md (LTO analysis)
-->

# Zero-cost performance

*This page is under construction. See the [benchmarks](./benchmarks.md) page
for current performance data.*

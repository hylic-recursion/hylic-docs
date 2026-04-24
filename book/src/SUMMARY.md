# Summary

[Introduction](./intro.md)
[Quick Start](./quickstart.md)
[Glossary](./glossary.md)

# Benchmarks

- [Results](./cookbook/benchmarks.md)

# Concepts

- [The recursive pattern](./concepts/separation.md)
- [The three domains](./concepts/domains.md)
- [Transforms and variance](./concepts/transforms.md)
- [Lifts — cross-axis transforms](./concepts/lifts.md)

# Guides

- [Fold: shaping the computation](./guides/fold.md)
- [Graph: controlling traversal](./guides/treeish.md)
- [Choosing an executor](./guides/execution.md)

# Pipelines

- [Overview](./pipeline/overview.md)
- [Stage 1 — SeedPipeline](./pipeline/seed.md)
- [Stage 1 — TreeishPipeline](./pipeline/treeish.md)
- [Stage 2 — LiftedPipeline](./pipeline/lifted.md)
- [Blanket sugar traits](./pipeline/sugars.md)
- [One-shot — OwnedPipeline](./pipeline/owned.md)
- [Writing a custom Lift](./pipeline/custom_lift.md)

# Executor (Funnel deep-dive)

- [The Exec pattern](./executor-design/exec_pattern.md)
- [Domain integration](./executor-design/domain_integration.md)
- [Policy traits](./executor-design/policy_traits.md)
- [Funnel overview](./funnel/overview.md)
- [Policies and presets](./funnel/policies.md)
- [CPS walk](./funnel/cps_walk.md)
- [Continuations](./funnel/continuations.md)
- [Cascade](./funnel/cascade.md)
- [Ticket system](./funnel/ticket_system.md)
- [Pool and dispatch](./funnel/pool_dispatch.md)
- [Queue strategies](./funnel/queue_strategies.md)
- [Accumulation](./funnel/accumulation.md)
- [Infrastructure](./funnel/infrastructure.md)
- [Testing](./funnel/testing.md)

# Cookbook

- [Fibonacci](./cookbook/fibonacci.md)
- [Expression evaluation](./cookbook/expression_eval.md)
- [Filesystem summary](./cookbook/filesystem_summary.md)
- [Cycle detection](./cookbook/cycle_detection.md)
- [Configuration inheritance](./cookbook/config_inheritance.md)
- [Parallel execution](./cookbook/parallel_execution.md)
- [Zero-cost performance](./cookbook/zero_cost_performance.md)
- [Transformations](./cookbook/transformations.md)
- [Module resolution](./cookbook/module_resolution.md)
- [Explainer (case study)](./cookbook/explainer.md)

# Reference

- [Import patterns](./guides/imports.md)
- [Concept map](./guides/concept_map.md)
- [Domain system (legacy design)](./design/domains.md)
- [Implementation notes](./design/implementation_notes.md)
- [Theory](./design/theory.md)
- [Algebra factorization](./design/milewski.md)
- [Pipeline transformability](./design/pipeline_transformability.md)

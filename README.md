# hylic-docs

Source for the [hylic documentation site](https://hylic-recursion.github.io/hylic-docs/). The site rebuilds and publishes from this repo's `master` branch via `.github/workflows/pages.yml`.

The book covers three crates from the hylic family. From [`hylic`](https://github.com/hylic-recursion/hylic) it has the [recursive pattern](https://hylic-recursion.github.io/hylic-docs/concepts/separation.html) and per-piece guides ([fold](https://hylic-recursion.github.io/hylic-docs/guides/fold.html), [treeish](https://hylic-recursion.github.io/hylic-docs/guides/treeish.html), [executor](https://hylic-recursion.github.io/hylic-docs/guides/execution.html)), an introduction to [lifts](https://hylic-recursion.github.io/hylic-docs/concepts/lifts.html), and a worked-example cookbook. From [`hylic-pipeline`](https://github.com/hylic-recursion/hylic-pipeline) it has the [pipeline overview](https://hylic-recursion.github.io/hylic-docs/pipeline/overview.html), the [sugar catalogue](https://hylic-recursion.github.io/hylic-docs/pipeline/sugars.html), and a chapter on [writing custom lifts](https://hylic-recursion.github.io/hylic-docs/pipeline/custom_lift.html). The [Funnel deep-dive](https://hylic-recursion.github.io/hylic-docs/funnel/overview.html) covers the parallel executor in detail: the CPS walk, defunctionalised continuations, ticket system, pool dispatch, and per-axis chapters on queue topology, accumulation, and wake. Benchmark results from [`hylic-benchmark`](https://github.com/hylic-recursion/hylic-benchmark) are rendered with an [interactive viewer](https://hylic-recursion.github.io/hylic-docs/cookbook/benchmarks.html) that marginalises on the policy matrix.

Code samples in the book are pulled from `src/docs_examples.rs` and the per-recipe files under `src/cookbook/` via mdBook `{{#include}}` directives. `cargo test -p hylic-docs` compiles and runs every snippet the book embeds, so the rendered code is always type-correct against the version of `hylic` and `hylic-pipeline` the workspace points at.

## Building locally

```bash
cd book && mdbook build      # → ../target/book/
cd book && mdbook serve      # preview at http://localhost:3000/
```

`mdbook` and `mdbook-graphviz` are required, plus `graphviz` at the system level. A local build expects the workspace layout (`hylic/` and `hylic-pipeline/` next to this crate), since `{{#include}}` paths reach into sibling crates. Not published to crates.io.

## License

Licensed under the [MIT License](./LICENSE).

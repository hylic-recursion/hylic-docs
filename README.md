# hylic-docs

Source for the
[hylic documentation site](https://hylic-recursion.github.io/hylic-docs/).
The site rebuilds and publishes from this repo's master branch
via `.github/workflows/pages.yml`.

The book covers:

- The fold / treeish / executor decomposition from
  [`hylic`](https://github.com/hylic-recursion/hylic) —
  [concepts](https://hylic-recursion.github.io/hylic-docs/concepts/separation.html),
  guides for each axis ([fold](https://hylic-recursion.github.io/hylic-docs/guides/fold.html),
  [treeish](https://hylic-recursion.github.io/hylic-docs/guides/treeish.html),
  [executor](https://hylic-recursion.github.io/hylic-docs/guides/execution.html)),
  and a worked-example cookbook.
- [`hylic-pipeline`](https://github.com/hylic-recursion/hylic-pipeline)'s
  chainable typestate, including the
  [sugar catalogue](https://hylic-recursion.github.io/hylic-docs/pipeline/sugars.html)
  and a guide to
  [writing custom lifts](https://hylic-recursion.github.io/hylic-docs/pipeline/custom_lift.html).
- The
  [Funnel executor](https://hylic-recursion.github.io/hylic-docs/funnel/overview.html)
  — CPS walk, continuations, three monomorphised policy axes,
  ticket system, pool dispatch.
- Benchmark results from
  [`hylic-benchmark`](https://github.com/hylic-recursion/hylic-benchmark),
  with an
  [interactive viewer](https://hylic-recursion.github.io/hylic-docs/cookbook/benchmarks.html)
  over the policy matrix.

## Building locally

```bash
cd book && mdbook build      # → ../target/book/
cd book && mdbook serve      # preview at http://localhost:3000/
```

`mdbook` and `mdbook-graphviz` are required; `graphviz` at the
system level. Code samples use `{{#include}}` directives that
pull from sibling crates, so a local build expects the
workspace layout (`hylic/` and `hylic-pipeline/` next to this
crate).

A `src/` directory holds doc-tested code samples;
`cargo test -p hylic-docs` compiles every snippet the book
embeds. Internal: not published to crates.io.

## License

Licensed under the [MIT License](./LICENSE).

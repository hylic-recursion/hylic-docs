# hylic-docs

Source for the
[hylic documentation site](https://hylic-recursion.github.io/hylic-docs/).

mdBook sources live under `book/`. The site rebuilds and
publishes to GitHub Pages on every push to master via
`.github/workflows/pages.yml`.

The site covers
[`hylic`](https://github.com/hylic-recursion/hylic) (the fold /
treeish / executor decomposition) and
[`hylic-pipeline`](https://github.com/hylic-recursion/hylic-pipeline)
(chainable typestate over hylic), the Funnel executor's design,
an interactive benchmark viewer, and a worked-example cookbook.

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

A `src/` directory holds doc-tested code samples; `cargo test
-p hylic-docs` compiles every snippet the book embeds.
Internal: not published to crates.io.

## License

Licensed under the [MIT License](./LICENSE).

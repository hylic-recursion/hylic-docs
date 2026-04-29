# hylic-docs

mdBook source for the hylic documentation site.

The published book covers the [`hylic`](../hylic/) core,
[`hylic-pipeline`](../hylic-pipeline/) typestate builder, the Funnel
executor's policy axes, and the cookbook of recipes.

## Layout

```
book/                 ← mdBook source (SUMMARY.md, src/, theme/)
src/                  ← Rust crate that holds doc-tested code samples
                       (cargo test -p hylic-docs covers the examples)
target/book/          ← `mdbook build` output (gitignored)
Makefile              ← `make hylic-docs-build`, `…serve`, `…check-anchors`
```

## Building

From this directory:

```bash
cd book && mdbook build      # output → target/book/
cd book && mdbook serve      # local preview at http://localhost:3000/
```

`mdbook` and `mdbook-graphviz` must be installed
(`cargo install --locked mdbook mdbook-graphviz`); `graphviz` itself
is required at the system level.

The book embeds `{{#include ../../../hylic*/src/...}}` snippets that
get type-checked when `cargo test -p hylic-docs --lib` runs.

## Status

Auxiliary crate. Not published to crates.io. The mdBook output may
be deployed to GitHub Pages.

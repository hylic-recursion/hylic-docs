# Filesystem summary

Aggregate file sizes, counts, and directory depth across a tree.

> **Imports:** `hylic::fold::simple_fold`, `hylic::graph::treeish_visit`, `hylic::cata::Strategy`

```rust
{{#include ../../../src/cookbook/filesystem_summary.rs}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__filesystem_summary__tests__fs_summary.snap:5:}}
```

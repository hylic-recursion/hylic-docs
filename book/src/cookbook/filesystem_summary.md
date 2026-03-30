# Filesystem summary

Fold a directory tree to compute total size, file count, and directory count.
The heap is a structured `Summary` that accumulates multiple metrics in one pass.

```rust
{{#include ../../../src/cookbook/filesystem_summary.rs:filesystem_summary}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__filesystem_summary__tests__filesystem_summary_result.snap:5:}}
```

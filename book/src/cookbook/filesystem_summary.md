# Filesystem summary

Aggregate file sizes, counts, and directory depth in one pass.
The heap is a structured `Summary` — multiple metrics accumulated
simultaneously.

```rust
{{#include ../../../src/cookbook/filesystem_summary.rs}}
```

Output:

```
{{#include ../../../src/cookbook/snapshots/hylic_docs__cookbook__filesystem_summary__tests__fs_summary.snap:5:}}
```

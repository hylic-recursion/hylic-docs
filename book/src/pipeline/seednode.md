# `SeedNode<N>` — the seed-rooted row type

```rust
{{#include ../../../../hylic/src/ops/lift/seed_node.rs:seed_node_enum}}
```

`SeedNode<N>` is the chain's input node type once `SeedLift`
has fired in a seed-rooted `Stage2Pipeline`. Two inhabitants:

- **`Node(N)`** — a real grown node from the user's seed graph.
- **`EntryRoot`** — the synthetic forest root above the entry
  seeds.

Variants are sealed; pattern-matching is not exposed to user
code. Inspection is through accessor methods:

| method                                      | returns                                          |
|----------------------------------------------|--------------------------------------------------|
| `sn.is_entry_root()`                         | `bool`                                           |
| `sn.as_node()`                               | `Option<&N>`                                     |
| `sn.into_node()`                             | `Option<N>`                                      |
| `sn.map_node(f: FnOnce(&N) -> M)`            | `SeedNode<M>` — Node mapped, EntryRoot preserved |

Inside Stage-2 sugar bodies, `SeedNode<N>` never appears; user
closures type at `&N` and the row is peeled (or routed past)
by [Wrap dispatch](./wrap_dispatch.md).

## Where the row surfaces

A lift whose output type mentions the chain's N can carry
`SeedNode<N>` to the chain tip. The explainer is the canonical
case:

```rust,ignore
let raw: ExplainerResult<SeedNode<N>, H, R> = pipeline
    .lift()
    .explain()
    .run_from_slice(&exec, &seeds, h0);
```

`ExplainerResult`'s first parameter is the per-node `heap.node`,
which on a seed-rooted chain is `SeedNode<N>`.

For walks over the trace (formatting, post-fact analysis),
project to an N-typed view via
[`SeedExplainerResult::from`](../../../../hylic/src/prelude/explainer.rs):

```rust,ignore
let sealed: SeedExplainerResult<N, H, R> = raw.into();
// sealed.entry_initial_heap, sealed.entry_working_heap, sealed.orig_result
//   — the EntryRoot row, promoted out of the tree as fields.
// sealed.roots: Vec<ExplainerResult<N, H, R>>
//   — per-seed subtrees, every node now plain N.
```

The conversion is total: every node below the EntryRoot row is
unwrapped, and `SeedNode<N>` no longer appears in the
user-visible shape.

## Tree shape

```text
        EntryRoot
        ├── Node(grow(seed_0))
        ├── Node(grow(seed_1))
        └── …
```

`SeedLift` produces this tree at run time from the entry seeds
and the user's `grow`. Each `Node(n)` below has the user's
`seeds_from_node + grow` as its own children-producing function.

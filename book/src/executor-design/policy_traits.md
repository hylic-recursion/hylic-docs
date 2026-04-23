# Policy Traits: Zero-Cost Configuration

Funnel's three behavioral axes (queue, accumulation, wake) are each
a trait with an associated `Spec` type. The `FunnelPolicy` bundle
combines them into one type parameter. This pattern —
**Spec → Store/State → Handle, resolved at compile time** — is the
general recipe for adding zero-overhead configuration axes to any
executor.

This page describes the pattern generically. For the concrete
implementations (Chase-Lev deques, streaming sweep, etc.), see the
[Funnel](../funnel/overview.md) section.

## Specs as data

Every Spec in hylic is `Copy` — a small value type that fully
describes configuration. This follows from the
[defunctionalization principle](exec_pattern.md): Specs are data,
not behavior. Combining Specs via axis transformations produces new
Specs. Attaching a resource to a Spec produces a Session. Running
a Spec creates the resource internally.

The policy sub-specs (`PerWorkerSpec`, `OnFinalizeSpec`, `EveryKSpec`,
etc.) are all `Copy + Default + Send + Sync`. Most are ZSTs. The
funnel `Spec<P>` composes them and is itself Copy (~40 bytes of
usizes and ZSTs).

## The Spec → Store → Handle pattern

Each axis follows the same three-phase lifecycle:

```dot process
digraph {
  rankdir=TB;
  node [shape=box, fontname="sans-serif", fontsize=10, style="rounded,filled"];
  edge [fontname="sans-serif", fontsize=9];

  trait_ [label="Trait\n(e.g. WorkStealing)\nassociated types:\nSpec, Store, Handle", fillcolor="#d4edda"];
  spec [label="Spec\nconstruction config\n(Copy + Default)", fillcolor="#fff3cd"];
  store [label="Store<N, H, R>\nper-fold resources\n(Send + Sync)", fillcolor="#cce5ff"];
  handle [label="Handle<'a, N, H, R>\nper-worker view\n(borrows Store)", fillcolor="#f8d7da"];

  trait_ -> spec [label="type Spec"];
  trait_ -> store [label="type Store"];
  trait_ -> handle [label="type Handle"];
  spec -> store [label="create_store()"];
  store -> handle [label="handle()"];
}
```

Three associated types capture the lifecycle:

1. **Spec** — construction-time configuration. Carried in the
   executor's `Spec<P>`. Small, Copy, Default.
2. **Store** — per-fold resources created from the Spec. Owned by
   the fold's stack frame. Send+Sync (shared across workers).
3. **Handle** — per-worker view that borrows from the Store. Has
   the actual push/pop/steal methods.

All three use GATs to carry the task's generic parameters without
boxing.

## Concrete example: WorkStealing

```rust
{{#include ../../../../hylic/src/exec/variant/funnel/policy/queue/mod.rs:work_stealing_trait}}
```

Two implementations:

| | PerWorker | Shared |
|---|---|---|
| **Spec** | `PerWorkerSpec { deque_capacity }` (Copy) | `SharedSpec` (ZST, Copy) |
| **Store** | `Vec<WorkerDeque>` + `AtomicU64` bitmask | `StealQueue` |
| **Handle** | refs to own deque + all deques + bitmask | ref to queue |

## Bundling: FunnelPolicy

Three independent axes combined into one type parameter:

```rust
{{#include ../../../../hylic/src/exec/variant/funnel/policy/mod.rs:funnel_policy_trait}}
```

```rust
{{#include ../../../../hylic/src/exec/variant/funnel/policy/mod.rs:policy_struct}}
```

`Policy<Q, A, W>` is the generic implementor. Named presets are type
aliases. The funnel `Spec<P>` carries each axis's sub-spec:

```rust
{{#include ../../../../hylic/src/exec/variant/funnel/mod.rs:funnel_spec}}
```

## Named presets as transformations

Every named preset is a transformation of `Spec::default(n)`.
Default values live in ONE place — the `default()` constructor.
Presets compose axis builders on top:

```rust
// WideLight = default + Shared queue + OnArrival accumulation
fn for_wide_light(n: usize) -> Spec<WideLight> {
    Spec::default(n)
        .with_queue::<Shared>(SharedSpec)
        .with_accumulate::<OnArrival>(OnArrivalSpec)
}
```

The axis builders (`with_queue`, `with_accumulate`, `with_wake`)
are typestate transformations — they change the Policy type parameter,
producing a new Spec type.

## How monomorphization flows

The type parameter propagates from Spec to every call site:

```dot process
digraph {
  rankdir=LR;
  node [shape=box, fontname="monospace", fontsize=9, style="rounded,filled"];
  edge [fontname="sans-serif", fontsize=8];

  user [label="Spec<WideLight>\n= Spec<Policy<Shared,\n  OnArrival, EveryPush>>", fillcolor="#fff3cd"];
  run [label=".run()\nroutes through\nwith_session", fillcolor="#f5f5f5"];
  store [label="P::Queue::create_store()\n= SharedStore", fillcolor="#cce5ff"];
  handle [label="P::Queue::handle()\n= SharedHandle", fillcolor="#cce5ff"];
  push [label="handle.push(task)\n= SharedHandle::push\n(direct call)", fillcolor="#d4edda"];
  deliver [label="P::Accumulate::deliver()\n= OnArrival::deliver\n(direct call)", fillcolor="#d4edda"];
  notify [label="P::Wake::should_notify()\n= EveryPush::should_notify\n(returns true)", fillcolor="#d4edda"];

  user -> run -> store -> handle -> push;
  run -> deliver;
  push -> notify;
}
```

From `Spec<WideLight>` to the innermost push/deliver/notify — every
call is resolved at compile time. No vtable, no trait object, no
indirect call.

## The const generic optimization

Wake strategies like `EveryK<K>` use a const generic for the
notification interval. The modulus `count % K` compiles to a bitmask
when K is a power of 2 — the compiler sees the constant and
optimizes.

## Applying the pattern to new axes

To add a fourth axis (e.g., steal ordering):

1. Define a trait: `pub trait StealOrder: 'static { type Spec: Copy + Default + Send + Sync; ... }`
2. Add implementations: `struct Fifo;`, `struct Lifo;`
3. Add to `FunnelPolicy`: `type Steal: StealOrder;`
4. Update `Policy<Q, A, W, St>` and named presets
5. Thread through `Spec<P>` and `run_fold`

The call chain monomorphizes automatically. No runtime cost for the
new axis.

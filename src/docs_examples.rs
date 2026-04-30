//! Typechecked code examples for documentation.
//!
//! Every rust code block in the docs should be {{#include}}'d from this
//! file (via ANCHOR markers) or from actual library source. No inline
//! code fences in markdown — if it's in the docs, it compiles here.

#[cfg(test)]
#[allow(dead_code)] // Doc examples show representative fixture structs; not all fields are read by test assertions
mod tests {
    // ── concepts/separation.md examples ────────────────

    // ANCHOR: treeish_constructor
    #[test]
    fn treeish_constructor() {
        use hylic::prelude::*;

        #[derive(Clone)]
        struct Dir { name: String, size: u64, children: Vec<Dir> }

        let graph: Treeish<Dir> = treeish(|d: &Dir| d.children.clone());
        let root = Dir { name: "root".into(), size: 10, children: vec![] };
        assert_eq!(graph.apply(&root).len(), 0);
    }
    // ANCHOR_END: treeish_constructor

    // ANCHOR: simple_fold_example
    #[test]
    fn identity_finalize_fold_example() {
        use hylic::prelude::*;

        #[derive(Clone)]
        struct Dir { name: String, size: u64, children: Vec<Dir> }

        let graph: Treeish<Dir>      = treeish(|d: &Dir| d.children.clone());
        let sum:   Fold<Dir, u64, u64> = fold(
            |d: &Dir| d.size,
            |heap: &mut u64, child: &u64| *heap += child,
            |h: &u64| *h,
        );

        let tree = Dir {
            name: "root".into(), size: 10,
            children: vec![
                Dir { name: "a".into(), size: 5, children: vec![] },
                Dir { name: "b".into(), size: 3, children: vec![] },
            ],
        };
        assert_eq!(FUSED.run(&sum, &graph, &tree), 18);
    }
    // ANCHOR_END: simple_fold_example

    // ANCHOR: exec_usage
    #[test]
    fn exec_usage() {
        use hylic::prelude::*;

        #[derive(Clone)]
        struct N { val: u64, children: Vec<N> }

        let graph: Treeish<N> = treeish(|n: &N| n.children.clone());
        let f:     Fold<N, u64, u64> = fold(
            |n: &N| n.val,
            |h: &mut u64, c: &u64| *h += c,
            |h: &u64| *h,
        );
        let root = N { val: 1, children: vec![N { val: 2, children: vec![] }] };

        // Sequential:
        let r1: u64 = FUSED.run(&f, &graph, &root);
        // Parallel — same fold, same graph:
        let r2: u64 = exec(funnel::Spec::default(4)).run(&f, &graph, &root);
        assert_eq!(r1, r2);
    }
    // ANCHOR_END: exec_usage

    // ── concepts/transforms.md examples ────────────────

    // ANCHOR: fold_wrap_init
    #[test]
    fn fold_wrap_init() {
        use hylic::prelude::*;

        #[derive(Clone)]
        struct Dir { name: String, size: u64, children: Vec<Dir> }

        let graph: Treeish<Dir> = treeish(|d: &Dir| d.children.clone());
        let f:     Fold<Dir, u64, u64> = fold(
            |d: &Dir| d.size,
            |h: &mut u64, c: &u64| *h += c,
            |h: &u64| *h,
        );

        let logged: Fold<Dir, u64, u64> = f.wrap_init(
            |d: &Dir, orig: &dyn Fn(&Dir) -> u64| orig(d),
        );

        let tree = Dir { name: "r".into(), size: 10, children: vec![] };
        assert_eq!(FUSED.run(&logged, &graph, &tree), 10);
    }
    // ANCHOR_END: fold_wrap_init

    // ANCHOR: fold_product
    #[test]
    fn fold_product() {
        use hylic::prelude::*;

        #[derive(Clone)]
        struct Dir { name: String, size: u64, children: Vec<Dir> }

        let graph: Treeish<Dir> = treeish(|d: &Dir| d.children.clone());
        let size_fold: Fold<Dir, u64, u64> = fold(
            |d: &Dir| d.size,
            |h: &mut u64, c: &u64| *h += c,
            |h: &u64| *h,
        );

        let both: Fold<Dir, (u64, usize), (u64, usize)> = size_fold.product(&depth_fold());
        let tree = Dir {
            name: "r".into(), size: 10,
            children: vec![Dir { name: "a".into(), size: 5, children: vec![] }],
        };
        let (total_size, max_depth) = FUSED.run(&both, &graph, &tree);
        assert_eq!(total_size, 15);
        assert_eq!(max_depth, 2);
    }
    // ANCHOR_END: fold_product

    // ANCHOR: explainer_usage
    #[test]
    fn explainer_usage() {
        use hylic_pipeline::prelude::*;

        #[derive(Clone)]
        struct N { val: u64, children: Vec<N> }

        let f: Fold<N, u64, u64> = fold(
            |n: &N| n.val,
            |h: &mut u64, c: &u64| *h += c,
            |h: &u64| *h,
        );
        let root = N { val: 1, children: vec![N { val: 2, children: vec![] }] };

        let trace: ExplainerResult<N, u64, u64> =
            TreeishPipeline::new(treeish(|n: &N| n.children.clone()), &f)
                .lift()
                .then_lift(Shared::explainer_lift::<N, u64, u64>())
                .run_from_node(&FUSED, &root);
        assert_eq!(trace.orig_result, 3);
    }
    // ANCHOR_END: explainer_usage

    // ── guides/execution.md examples ───────────────────

    // ANCHOR: domain_switching
    #[test]
    fn domain_switching() {
        use hylic::domain::{shared as sdom, local as ldom, owned as odom};
        use hylic::graph::treeish_visit;

        #[derive(Clone)]
        struct N { val: u64, children: Vec<N> }

        // Same closures used to build a fold in each domain.
        let init = |n: &N| n.val;
        let acc  = |h: &mut u64, c: &u64| *h += c;
        let fin  = |h: &u64| *h;
        fn children(n: &N, cb: &mut dyn FnMut(&N)) {
            for c in &n.children { cb(c); }
        }

        let root = N { val: 1, children: vec![N { val: 2, children: vec![] }] };

        // Shared (Arc):
        let r1: u64 = sdom::FUSED.run(
            &sdom::fold(init, acc, fin),
            &treeish_visit(children),
            &root,
        );
        // Local (Rc):
        let r2: u64 = ldom::FUSED.run(
            &ldom::fold(init, acc, fin),
            &treeish_visit(children),
            &root,
        );
        // Owned (Box):
        let r3: u64 = odom::FUSED.run(
            &odom::fold(init, acc, fin),
            &treeish_visit(children),
            &root,
        );

        assert_eq!(r1, r2);
        assert_eq!(r2, r3);
    }
    // ANCHOR_END: domain_switching

    // ANCHOR: runtime_dispatch
    #[test]
    fn runtime_dispatch() {
        use hylic::prelude::*;

        #[derive(Clone)]
        struct N { val: u64, children: Vec<N> }

        let graph: Treeish<N> = treeish(|n: &N| n.children.clone());
        let f:     Fold<N, u64, u64> = fold(
            |n: &N| n.val,
            |h: &mut u64, c: &u64| *h += c,
            |h: &u64| *h,
        );
        let root = N { val: 1, children: vec![N { val: 2, children: vec![] }] };

        let r1: u64 = FUSED.run(&f, &graph, &root);
        let r2: u64 = exec(funnel::Spec::default(4)).run(&f, &graph, &root);
        assert_eq!(r1, r2);
        assert_eq!(r1, 3);
    }
    // ANCHOR_END: runtime_dispatch

    // ── guides/graph.md examples ───────────────────────

    // ANCHOR: treeish_constructors
    #[test]
    fn treeish_constructors() {
        use hylic::prelude::*;

        #[derive(Clone)]
        struct Node { value: u64, children: Vec<Node> }

        let root = Node { value: 1, children: vec![Node { value: 2, children: vec![] }] };

        // Callback-based (zero allocation per visit):
        let g1: Treeish<Node> = treeish_visit(|n: &Node, cb: &mut dyn FnMut(&Node)| {
            for child in &n.children { cb(child); }
        });

        // Vec-returning (allocates per visit):
        let g2: Treeish<Node> = treeish(|n: &Node| n.children.clone());

        // Slice accessor (borrows, zero allocation):
        let g3: Treeish<Node> = treeish_from(|n: &Node| n.children.as_slice());

        assert_eq!(g1.apply(&root).len(), 1);
        assert_eq!(g2.apply(&root).len(), 1);
        assert_eq!(g3.apply(&root).len(), 1);

        // Flat data — nodes are indices, children from adjacency list:
        let adj: Vec<Vec<usize>> = vec![vec![1, 2], vec![], vec![]];
        let g4: Treeish<usize> = treeish_visit(move |n: &usize, cb: &mut dyn FnMut(&usize)| {
            for &c in &adj[*n] { cb(&c); }
        });
        assert_eq!(g4.apply(&0).len(), 2);
    }
    // ANCHOR_END: treeish_constructors

    // ANCHOR: graph_filter
    #[test]
    fn graph_filter() {
        use hylic::prelude::*;

        #[derive(Clone)]
        struct Node { value: u64, children: Vec<Node> }

        let graph: Treeish<Node> = treeish(|n: &Node| n.children.clone());
        let f:     Fold<Node, u64, u64> = fold(
            |n: &Node| n.value,
            |h: &mut u64, c: &u64| *h += c,
            |h: &u64| *h,
        );

        let root = Node { value: 1, children: vec![
            Node { value: 10, children: vec![] },
            Node { value: 2, children: vec![] },
        ]};

        // Only visit children with value > 5.
        let pruned: Treeish<Node> = graph.filter(|child: &Node| child.value > 5);
        let result: u64 = FUSED.run(&f, &pruned, &root);
        assert_eq!(result, 11); // 1 + 10 (skipped 2)
    }
    // ANCHOR_END: graph_filter

    // ANCHOR: memoize_example
    #[test]
    fn memoize_example() {
        use hylic::prelude::*;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let call_count = Arc::new(AtomicUsize::new(0));
        let cc = call_count.clone();

        let graph: Treeish<u64> = treeish(move |n: &u64| -> Vec<u64> {
            cc.fetch_add(1, Ordering::Relaxed);
            if *n == 0 { vec![] } else { vec![n - 1] }
        });
        let f: Fold<u64, u64, u64> = fold(
            |n: &u64| *n,
            |h: &mut u64, c: &u64| *h += c,
            |h: &u64| *h,
        );

        let cached: Treeish<u64> = memoize_treeish(&graph);
        let _ = FUSED.run(&f, &cached, &3u64);
        let first_count = call_count.load(Ordering::Relaxed);

        // Second run hits the cache; no new calls into `graph`.
        let _ = FUSED.run(&f, &cached, &3u64);
        let second_count = call_count.load(Ordering::Relaxed);
        assert_eq!(first_count, second_count);
    }
    // ANCHOR_END: memoize_example

    // ── guides/fold.md examples ────────────────────────

    // ANCHOR: named_closures_pattern
    #[test]
    fn named_closures_pattern() {
        use hylic::prelude::*;

        #[derive(Clone)]
        struct N { val: u64, children: Vec<N> }

        // Named closures — reusable across domains.
        let init = |n: &N| n.val;
        let acc  = |h: &mut u64, c: &u64| *h += c;
        let fin  = |h: &u64| *h;

        // A free `fn` makes the visit function's full type visible at the
        // binding site; capture-free, reusable across domains.
        fn children(n: &N, cb: &mut dyn FnMut(&N)) {
            for c in &n.children { cb(c); }
        }

        let f:     Fold<N, u64, u64> = fold(init, acc, fin);
        let graph: Treeish<N>        = treeish_visit(children);
        let root = N { val: 1, children: vec![N { val: 2, children: vec![] }] };

        assert_eq!(FUSED.run(&f, &graph, &root), 3);
    }
    // ANCHOR_END: named_closures_pattern

    // ANCHOR: fold_zipmap
    #[test]
    fn fold_zipmap() {
        use hylic::prelude::*;

        #[derive(Clone)]
        struct N { val: u64, children: Vec<N> }

        let graph: Treeish<N> = treeish(|n: &N| n.children.clone());
        let f:     Fold<N, u64, u64> = fold(
            |n: &N| n.val,
            |h: &mut u64, c: &u64| *h += c,
            |h: &u64| *h,
        );

        let with_flag: Fold<N, u64, (u64, bool)> = f.zipmap(|r: &u64| *r > 5);
        let root = N { val: 1, children: vec![
            N { val: 3, children: vec![] },
            N { val: 4, children: vec![] },
        ]};
        let (total, over_five): (u64, bool) = FUSED.run(&with_flag, &graph, &root);
        assert_eq!(total, 8);
        assert!(over_five);
    }
    // ANCHOR_END: fold_zipmap

    // ANCHOR: fold_contramap
    #[test]
    fn fold_contramap() {
        use hylic::prelude::*;

        #[derive(Clone)]
        struct N { val: u64, children: Vec<N> }

        let f: Fold<N, u64, u64> = fold(
            |n: &N| n.val,
            |h: &mut u64, c: &u64| *h += c,
            |h: &u64| *h,
        );

        // Change node type: String → N.
        let by_name: Fold<String, u64, u64> =
            f.contramap_n(|s: &String| N { val: s.len() as u64, children: vec![] });
        let graph: Treeish<String> =
            treeish_visit(|_: &String, _cb: &mut dyn FnMut(&String)| {});

        let result: u64 = FUSED.run(&by_name, &graph, &"hello".to_string());
        assert_eq!(result, 5);
    }
    // ANCHOR_END: fold_contramap

    // ── intro.md ───────────────────────────────────────

    // ANCHOR: intro_dir_example
    #[test]
    fn intro_dir_example() {
        use hylic::prelude::*;

        #[derive(Clone)]
        struct Dir { name: String, size: u64, children: Vec<Dir> }

        let graph = treeish(|d: &Dir| d.children.clone());
        let fold = fold(
            |d: &Dir| d.size,
            |heap: &mut u64, child: &u64| *heap += child,
            |heap: &u64| *heap,
        );

        let tree = Dir {
            name: "project".into(), size: 10,
            children: vec![
                Dir { name: "src".into(), size: 200, children: vec![] },
                Dir { name: "docs".into(), size: 50, children: vec![] },
            ],
        };

        // Sequential:
        let total = FUSED.run(&fold, &graph, &tree);
        assert_eq!(total, 260);

        // Parallel — same fold, same graph:
        let total = exec(funnel::Spec::default(4)).run(&fold, &graph, &tree);
        assert_eq!(total, 260);
    }
    // ANCHOR_END: intro_dir_example

    // ANCHOR: intro_flat_example
    #[test]
    fn intro_flat_example() {
        use hylic::prelude::*;

        // Flat adjacency list — nodes are indices, children are looked up
        let children: Vec<Vec<usize>> = vec![
            vec![1, 2],  // node 0 → children 1, 2
            vec![],      // node 1 → leaf
            vec![],      // node 2 → leaf
        ];
        let graph = treeish_visit(move |n: &usize, cb: &mut dyn FnMut(&usize)| {
            for &c in &children[*n] { cb(&c); }
        });
        let fold = fold(|n: &usize| *n as u64, |h: &mut u64, c: &u64| *h += c, |h| h.clone());

        let total = FUSED.run(&fold, &graph, &0);
        assert_eq!(total, 3); // 0 + 1 + 2
    }
    // ANCHOR_END: intro_flat_example

    // ── quickstart.md ─────────────────────────────────

    // ANCHOR: quickstart_funnel
    #[test]
    fn quickstart_funnel() {
        use hylic::prelude::*;

        #[derive(Clone)]
        struct Dir { name: String, size: u64, children: Vec<Dir> }

        let graph = treeish(|d: &Dir| d.children.clone());
        let fold = fold(
            |d: &Dir| d.size,
            |heap: &mut u64, child: &u64| *heap += child,
            |heap: &u64| *heap,
        );

        let tree = Dir {
            name: "root".into(), size: 10,
            children: vec![
                Dir { name: "a".into(), size: 5, children: vec![] },
                Dir { name: "b".into(), size: 3, children: vec![] },
            ],
        };

        let total = exec(funnel::Spec::default(4)).run(&fold, &graph, &tree);
        assert_eq!(total, 18);
    }
    // ANCHOR_END: quickstart_funnel

    // ANCHOR: quickstart_session
    #[test]
    fn quickstart_session() {
        use hylic::prelude::*;

        #[derive(Clone)]
        struct Dir { name: String, size: u64, children: Vec<Dir> }

        let graph = treeish(|d: &Dir| d.children.clone());
        let fold = fold(
            |d: &Dir| d.size,
            |heap: &mut u64, child: &u64| *heap += child,
            |heap: &u64| *heap,
        );

        let tree = Dir {
            name: "root".into(), size: 10,
            children: vec![
                Dir { name: "a".into(), size: 5, children: vec![] },
            ],
        };

        exec(funnel::Spec::default(4)).session(|s| {
            let r1 = s.run(&fold, &graph, &tree);
            let r2 = s.run(&fold, &graph, &tree);
            assert_eq!(r1, r2);
        });
    }
    // ANCHOR_END: quickstart_session

    // ── guides/seed_pipeline.md ───────────────────────

    // ANCHOR: seed_pipeline_example
    #[test]
    fn seed_pipeline_example() {
        use hylic_pipeline::prelude::*;
        use std::collections::HashMap;

        // The "registry" — flat data, not a tree.
        let mut modules: HashMap<String, Vec<String>> = HashMap::new();
        modules.insert("app".into(),  vec!["db".into(), "auth".into()]);
        modules.insert("db".into(),   vec![]);
        modules.insert("auth".into(), vec!["db".into()]);

        // Edge function: given a module name, produce its dependency seeds.
        let reg = modules.clone();
        let seeds_from_node: Edgy<String, String> =
            edgy_visit(move |name: &String, cb: &mut dyn FnMut(&String)| {
                if let Some(deps) = reg.get(name) {
                    for dep in deps { cb(dep); }
                }
            });

        // Fold: collect every reachable name.
        let f: Fold<String, Vec<String>, Vec<String>> = fold(
            |name: &String| vec![name.clone()],
            |heap: &mut Vec<String>, child: &Vec<String>| heap.extend(child.iter().cloned()),
            |heap: &Vec<String>| heap.clone(),
        );

        let pipeline: SeedPipeline<Shared, String, String, Vec<String>, Vec<String>> =
            SeedPipeline::new(|seed: &String| seed.clone(), seeds_from_node, &f);

        let result: Vec<String> = pipeline.run_from_slice(
            &FUSED,
            &["app".to_string()],
            Vec::new(),
        );
        assert!(result.contains(&"app".to_string()));
        assert!(result.contains(&"auth".to_string()));
    }
    // ANCHOR_END: seed_pipeline_example

    // ANCHOR: seed_pipeline_parallel
    #[test]
    fn seed_pipeline_parallel() {
        use hylic_pipeline::prelude::*;
        use std::collections::HashMap;

        let mut modules: HashMap<String, Vec<String>> = HashMap::new();
        modules.insert("app".into(),  vec!["db".into(), "auth".into()]);
        modules.insert("db".into(),   vec![]);
        modules.insert("auth".into(), vec!["db".into()]);

        let reg = modules.clone();
        let seeds_from_node: Edgy<String, String> =
            edgy_visit(move |name: &String, cb: &mut dyn FnMut(&String)| {
                if let Some(deps) = reg.get(name) { for dep in deps { cb(dep); } }
            });

        let f: Fold<String, Vec<String>, Vec<String>> = fold(
            |name: &String| vec![name.clone()],
            |heap: &mut Vec<String>, child: &Vec<String>| heap.extend(child.iter().cloned()),
            |heap: &Vec<String>| heap.clone(),
        );

        let pipeline: SeedPipeline<Shared, String, String, Vec<String>, Vec<String>> =
            SeedPipeline::new(|seed: &String| seed.clone(), seeds_from_node, &f);

        let result: Vec<String> = pipeline.run_from_slice(
            &exec(funnel::Spec::default(4)),
            &["app".to_string()],
            Vec::new(),
        );
        assert!(result.contains(&"app".to_string()));
    }
    // ANCHOR_END: seed_pipeline_parallel

    // ── concepts/domains.md ──────────────────────────────

    // ANCHOR: domains_three_folds
    #[test]
    fn domains_three_folds() {
        // Shared: closures must be Send + Sync (they go into Arc).
        let _shared = hylic::domain::shared::fold(
            |n: &u64| *n,                     // init
            |h: &mut u64, c: &u64| *h += c,   // accumulate
            |h: &u64| *h,                     // finalize
        );

        // Local: closures can capture Rc / RefCell.
        use std::cell::RefCell;
        use std::rc::Rc;
        let state = Rc::new(RefCell::new(0u32));
        let state_for_init = state.clone();
        let _local = hylic::domain::local::fold(
            move |n: &u64| { *state_for_init.borrow_mut() += 1; *n },
            |h: &mut u64, c: &u64| *h += c,
            |h: &u64| *h,
        );

        // Owned: one-shot construction; not Clone.
        let _owned = hylic::domain::owned::fold(
            |n: &u64| *n,
            |h: &mut u64, c: &u64| *h += c,
            |h: &u64| *h,
        );
    }
    // ANCHOR_END: domains_three_folds

    // ── concepts/lifts.md ──────────────────────────────

    // ANCHOR: bare_lift_wrap_init
    #[test]
    fn bare_lift_wrap_init() {
        use hylic::prelude::*;

        let t:   Treeish<u64>      = treeish(|n: &u64| if *n > 0 { vec![*n - 1] } else { vec![] });
        let fld: Fold<u64, u64, u64> = fold(|n: &u64| *n, |h: &mut u64, c: &u64| *h += c, |h: &u64| *h);

        // Wrap init to add +1 at each node.
        let wi = Shared::wrap_init_lift::<u64, u64, u64, _>(|n, orig| orig(n) + 1);
        let r:  u64 = wi.run_on(&FUSED, t, fld, &3u64);
        // Tree 3→2→1→0: 4 nodes, each +1 → 4 extra → 6 + 4 = 10.
        assert_eq!(r, 10);
    }
    // ANCHOR_END: bare_lift_wrap_init

    // ── pipeline/overview.md ─────────────────────────────

    // ANCHOR: pipeline_overview_treeish
    #[test]
    fn pipeline_overview_treeish() {
        use hylic_pipeline::prelude::*;

        #[derive(Clone)]
        struct Node { value: u64, children: Vec<Node> }
        let root = Node {
            value: 1,
            children: vec![
                Node { value: 2, children: vec![] },
                Node { value: 3, children: vec![] },
            ],
        };

        let tp: TreeishPipeline<Shared, Node, u64, u64> = TreeishPipeline::new(
            treeish(|n: &Node| n.children.clone()),
            &fold(|n: &Node| n.value, |h: &mut u64, c: &u64| *h += c, |h: &u64| *h),
        );

        let r: (u64, bool) = tp
            .wrap_init(|n: &Node, orig: &dyn Fn(&Node) -> u64| orig(n) + 1)
            .zipmap(|r: &u64| *r > 5)
            .run_from_node(&FUSED, &root);

        // init+1 on 3 nodes → 6 + 3 = 9; (9, true).
        assert_eq!(r, (9, true));
    }
    // ANCHOR_END: pipeline_overview_treeish

    // ANCHOR: pipeline_overview_seed
    #[test]
    fn pipeline_overview_seed() {
        use hylic_pipeline::prelude::*;
        use std::collections::HashMap;
        use std::sync::Arc;

        #[derive(Clone)]
        struct Mod { cost: u64, deps: Vec<String> }
        let reg: Arc<HashMap<String, Mod>> = Arc::new({
            let mut m = HashMap::new();
            m.insert("app".into(), Mod { cost: 1, deps: vec!["db".into()] });
            m.insert("db".into(),  Mod { cost: 2, deps: vec![] });
            m
        });
        let reg_grow = reg.clone();

        let sp: SeedPipeline<Shared, Mod, String, u64, u64> = SeedPipeline::new(
            move |s: &String| reg_grow.get(s).cloned().unwrap(),
            edgy_visit(|n: &Mod, cb: &mut dyn FnMut(&String)| {
                for d in &n.deps { cb(d); }
            }),
            &fold(|n: &Mod| n.cost, |h: &mut u64, c: &u64| *h += c, |h: &u64| *h),
        );

        let r: u64 = sp
            .filter_seeds(|s: &String| !s.starts_with('_'))
            .run_from_slice(&FUSED, &["app".to_string()], 0u64);

        // Reachable modules: app (cost 1) + db (cost 2) = 3.
        assert_eq!(r, 3);
    }
    // ANCHOR_END: pipeline_overview_seed

    // ── pipeline/treeish.md ──────────────────────────────

    // ANCHOR: treeish_pipeline_ctor
    #[test]
    fn treeish_pipeline_ctor() {
        use hylic_pipeline::prelude::*;

        #[derive(Clone)]
        struct Node { value: u64, children: Vec<Node> }
        let root = Node { value: 7, children: vec![] };

        let tp: TreeishPipeline<Shared, Node, u64, u64> = TreeishPipeline::new(
            treeish(|n: &Node| n.children.clone()),
            &fold(
                |n: &Node| n.value,
                |h: &mut u64, c: &u64| *h += c,
                |h: &u64| *h,
            ),
        );
        assert_eq!(tp.run_from_node(&FUSED, &root), 7);
    }
    // ANCHOR_END: treeish_pipeline_ctor

    // ANCHOR: treeish_pipeline_chain
    #[test]
    fn treeish_pipeline_chain() {
        use hylic_pipeline::prelude::*;

        #[derive(Clone)]
        struct Node { value: u64, children: Vec<Node> }
        let root = Node {
            value: 1,
            children: vec![
                Node { value: 2, children: vec![] },
                Node { value: 3, children: vec![] },
            ],
        };

        let tp: TreeishPipeline<Shared, Node, u64, u64> = TreeishPipeline::new(
            treeish(|n: &Node| n.children.clone()),
            &fold(|n: &Node| n.value, |h: &mut u64, c: &u64| *h += c, |h: &u64| *h),
        );

        let r: (u64, bool) = tp
            .wrap_init(|n: &Node, orig: &dyn Fn(&Node) -> u64| orig(n) + 1)
            .zipmap(|r: &u64| *r > 5)
            .run_from_node(&FUSED, &root);
        assert_eq!(r, (9, true));
    }
    // ANCHOR_END: treeish_pipeline_chain

    // ── pipeline/lifted.md ───────────────────────────────

    // ANCHOR: lifted_sugar_chain
    #[test]
    fn lifted_sugar_chain() {
        use hylic_pipeline::prelude::*;

        let tp: TreeishPipeline<Shared, u64, u64, u64> = TreeishPipeline::new(
            treeish(|n: &u64| if *n > 0 { vec![*n - 1] } else { vec![] }),
            &fold(|n: &u64| *n, |h: &mut u64, c: &u64| *h += c, |h: &u64| *h),
        );

        let r: String = tp
            .wrap_init(|n: &u64, orig: &dyn Fn(&u64) -> u64| orig(n) + 1)
            .zipmap(|r: &u64| *r > 5)
            .filter_edges(|n: &u64| *n != 0)
            .map_r_bi(
                |r: &(u64, bool)| format!("{}:{}", r.0, r.1),
                |s: &String| {
                    let (a, b) = s.split_once(':').unwrap();
                    (a.parse().unwrap(), b == "true")
                },
            )
            .run_from_node(&FUSED, &3u64);

        // filter_edges drops the 0-step: tree visits 3→2→1, three nodes.
        // wrap_init adds +1 each → values 4, 3, 2; sum = 9. zipmap > 5 → true.
        assert_eq!(r, "9:true");
    }
    // ANCHOR_END: lifted_sugar_chain

    // ── pipeline/owned.md ────────────────────────────────

    // ANCHOR: owned_pipeline_example
    #[test]
    fn owned_pipeline_example() {
        use hylic_pipeline::{OwnedPipeline, PipelineExecOnce};
        use hylic::domain::owned as odom;

        let graph = odom::edgy::treeish(|n: &u64|
            if *n > 0 { vec![*n - 1] } else { vec![] });
        let fld = odom::fold(
            |n: &u64| *n,
            |h: &mut u64, c: &u64| *h += c,
            |h: &u64| *h,
        );

        let r: u64 = OwnedPipeline::new(graph, fld)
            .run_from_node_once(&odom::FUSED, &5u64);
        // 5+4+3+2+1+0 = 15.
        assert_eq!(r, 15);
    }
    // ANCHOR_END: owned_pipeline_example

    // ── pipeline/custom_lift.md ──────────────────────────

    // ANCHOR: custom_lift_note_visits
    #[test]
    fn custom_lift_note_visits() {
        use std::sync::{Arc, Mutex};
        use hylic::domain::Shared;
        use hylic::domain::shared::fold::{self as sfold, Fold};
        use hylic::graph::Treeish;
        use hylic::ops::Lift;

        /// Counts init calls into a shared counter.
        #[derive(Clone)]
        struct NoteVisits {
            counter: Arc<Mutex<u64>>,
        }

        impl<N, H, R> Lift<Shared, N, H, R> for NoteVisits
        where N: Clone + 'static, H: Clone + 'static, R: Clone + 'static,
        {
            type N2   = N;
            type MapH = H;
            type MapR = R;

            fn apply<T>(
                &self,
                treeish: Treeish<N>,
                fold:    Fold<N, H, R>,
                cont: impl FnOnce(
                    Treeish<N>,
                    Fold<N, H, R>,
                ) -> T,
            ) -> T
            {
                let fold_for_init = fold.clone();
                let fold_for_acc  = fold.clone();
                let fold_for_fin  = fold;
                let counter       = self.counter.clone();
                let wrapped: Fold<N, H, R> = sfold::fold(
                    move |n: &N| { *counter.lock().unwrap() += 1; fold_for_init.init(n) },
                    move |h: &mut H, r: &R| fold_for_acc.accumulate(h, r),
                    move |h: &H| fold_for_fin.finalize(h),
                );
                cont(treeish, wrapped)
            }
        }

        use hylic::ops::LiftBare;
        use hylic::prelude::{treeish, fold, FUSED};

        let counter = Arc::new(Mutex::new(0u64));
        let lift    = NoteVisits { counter: counter.clone() };
        let t       = treeish(|n: &u64| if *n > 0 { vec![*n - 1] } else { vec![] });
        let f       = fold(|n: &u64| *n, |h: &mut u64, c: &u64| *h += c, |h: &u64| *h);
        let r: u64  = lift.run_on(&FUSED, t, f, &3u64);
        assert_eq!(r, 6);                            // 3 + 2 + 1 + 0
        assert_eq!(*counter.lock().unwrap(), 4);     // four init calls
    }
    // ANCHOR_END: custom_lift_note_visits

    // ── pipeline/explainer.md ────────────────────────────

    // ANCHOR: explainer_orig_result
    #[test]
    fn explainer_orig_result() {
        use hylic_pipeline::prelude::*;

        #[derive(Clone)]
        struct Node { v: u64, ch: Vec<Node> }
        let root = Node { v: 3, ch: vec![
            Node { v: 2, ch: vec![] },
            Node { v: 1, ch: vec![] },
        ]};

        let tp: TreeishPipeline<Shared, Node, u64, u64> = TreeishPipeline::new(
            treeish(|n: &Node| n.ch.clone()),
            &fold(|n: &Node| n.v, |h: &mut u64, c: &u64| *h += c, |h: &u64| *h),
        );

        let trace: ExplainerResult<Node, u64, u64> = tp
            .explain()
            .run_from_node(&FUSED, &root);
        // Sum = 3 + 2 + 1 = 6.
        assert_eq!(trace.orig_result, 6);
        // Every non-leaf records its child-accumulations.
        assert!(!trace.heap.transitions.is_empty());
    }
    // ANCHOR_END: explainer_orig_result

    // ── guides/bare_lift.md ──────────────────────────────

    // ANCHOR: bare_lift_composed
    #[test]
    fn bare_lift_composed() {
        use hylic::prelude::*;
        use hylic::ops::ComposedLift;

        let t:   Treeish<u64>        = treeish(|n: &u64| if *n > 0 { vec![*n - 1] } else { vec![] });
        let fld: Fold<u64, u64, u64> = fold(|n: &u64| *n, |h: &mut u64, c: &u64| *h += c, |h: &u64| *h);

        let l1 = Shared::wrap_init_lift::<u64, u64, u64, _>(|n, orig| orig(n) + 1);
        let l2 = Shared::zipmap_lift::<u64, u64, u64, bool, _>(|r: &u64| *r > 5);
        let composed = ComposedLift::compose(l1, l2);

        let (r, flag): (u64, bool) = composed.run_on(&FUSED, t, fld, &3u64);
        // wrap_init: 3+1+2+1+1+1+0+1 = 10; zipmap: (10, true).
        assert_eq!(r, 10);
        assert!(flag);
    }
    // ANCHOR_END: bare_lift_composed

    // ── cookbook/zero_cost_performance.md ───────────────

    // ANCHOR: zero_cost_treeops
    #[test]
    fn zero_cost_treeops() {
        use hylic::prelude::*;
        use hylic::ops::TreeOps;

        #[derive(Clone)]
        struct TreeNode { id: usize, value: u64 }

        struct AdjGraph {
            adj:   Vec<Vec<usize>>,
            nodes: Vec<TreeNode>,
        }
        impl TreeOps<TreeNode> for AdjGraph {
            fn visit(&self, node: &TreeNode, cb: &mut dyn FnMut(&TreeNode)) {
                for &child_id in &self.adj[node.id] {
                    cb(&self.nodes[child_id]);
                }
            }
        }

        let graph = AdjGraph {
            adj: vec![vec![1, 2], vec![], vec![]],
            nodes: vec![
                TreeNode { id: 0, value: 1 },
                TreeNode { id: 1, value: 2 },
                TreeNode { id: 2, value: 3 },
            ],
        };

        let f: Fold<TreeNode, u64, u64> = fold(
            |n: &TreeNode| n.value,
            |h: &mut u64, r: &u64| *h += r,
            |h: &u64| *h,
        );
        let root = graph.nodes[0].clone();
        let total: u64 = FUSED.run(&f, &graph, &root);
        assert_eq!(total, 6);
    }
    // ANCHOR_END: zero_cost_treeops
}

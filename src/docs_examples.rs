//! Typechecked code examples for documentation.
//!
//! Every rust code block in the docs should be {{#include}}'d from this
//! file (via ANCHOR markers) or from actual library source. No inline
//! code fences in markdown — if it's in the docs, it compiles here.

#[cfg(test)]
#[allow(dead_code)] // Doc examples show representative fixture structs; not all fields are read by test assertions
mod tests {
    use hylic::domain::shared as dom;
use hylic::graph;
    // ── concepts/separation.md examples ────────────────

    // ANCHOR: treeish_constructor
    #[test]
    fn treeish_constructor() {

        #[derive(Clone)]
        struct Dir { name: String, size: u64, children: Vec<Dir> }

        let graph = graph::treeish(|d: &Dir| d.children.clone());
        let root = Dir { name: "root".into(), size: 10, children: vec![] };
        assert_eq!(graph.apply(&root).len(), 0);
    }
    // ANCHOR_END: treeish_constructor

    // ANCHOR: simple_fold_example
    #[test]
    fn simple_fold_example() {

        #[derive(Clone)]
        struct Dir { name: String, size: u64, children: Vec<Dir> }

        let graph = graph::treeish(|d: &Dir| d.children.clone());
        let init = |d: &Dir| d.size;
        let acc = |heap: &mut u64, child: &u64| *heap += child;
        let sum = dom::simple_fold(init, acc);

        let tree = Dir {
            name: "root".into(), size: 10,
            children: vec![
                Dir { name: "a".into(), size: 5, children: vec![] },
                Dir { name: "b".into(), size: 3, children: vec![] },
            ],
        };
        assert_eq!(dom::FUSED.run(&sum, &graph, &tree), 18);
    }
    // ANCHOR_END: simple_fold_example

    // ANCHOR: exec_usage
    #[test]
    fn exec_usage() {
        use hylic::cata::exec::funnel;

        #[derive(Clone)]
        struct N { val: u64, children: Vec<N> }

        let graph = graph::treeish(|n: &N| n.children.clone());
        let init = |n: &N| n.val;
        let acc = |h: &mut u64, c: &u64| *h += c;
        let fold = dom::simple_fold(init, acc);
        let root = N { val: 1, children: vec![N { val: 2, children: vec![] }] };

        let r = dom::FUSED.run(&fold, &graph, &root);           // sequential
        let r2 = dom::exec(funnel::Spec::default(4)).run(&fold, &graph, &root); // parallel
        assert_eq!(r, r2);
    }
    // ANCHOR_END: exec_usage

    // ── concepts/transforms.md examples ────────────────

    // ANCHOR: fold_wrap_init
    #[test]
    fn fold_wrap_init() {

        #[derive(Clone)]
        struct Dir { name: String, size: u64, children: Vec<Dir> }

        let graph = graph::treeish(|d: &Dir| d.children.clone());
        let init = |d: &Dir| d.size;
        let acc = |h: &mut u64, c: &u64| *h += c;
        let fold = dom::simple_fold(init, acc);

        let logged = fold.wrap_init(|d: &Dir, orig: &dyn Fn(&Dir) -> u64| {
            // side effect: could log here
            orig(d)
        });

        let tree = Dir { name: "r".into(), size: 10, children: vec![] };
        assert_eq!(dom::FUSED.run(&logged, &graph, &tree), 10);
    }
    // ANCHOR_END: fold_wrap_init

    // ANCHOR: fold_product
    #[test]
    fn fold_product() {
        use hylic::prelude::depth_fold;

        #[derive(Clone)]
        struct Dir { name: String, size: u64, children: Vec<Dir> }

        let graph = graph::treeish(|d: &Dir| d.children.clone());
        let init = |d: &Dir| d.size;
        let acc = |h: &mut u64, c: &u64| *h += c;
        let size_fold = dom::simple_fold(init, acc);

        let both = size_fold.product(&depth_fold());
        let tree = Dir {
            name: "r".into(), size: 10,
            children: vec![Dir { name: "a".into(), size: 5, children: vec![] }],
        };
        let (total_size, max_depth) = dom::FUSED.run(&both, &graph, &tree);
        assert_eq!(total_size, 15);
        assert_eq!(max_depth, 2);
    }
    // ANCHOR_END: fold_product

    // ANCHOR: explainer_usage
    #[test]
    fn explainer_usage() {
        use hylic::domain::Shared;
        use hylic_pipeline::prelude::{TreeishPipeline, PipelineExec};

        #[derive(Clone)]
        struct N { val: u64, children: Vec<N> }

        let init = |n: &N| n.val;
        let acc = |h: &mut u64, c: &u64| *h += c;
        let fold_ = dom::simple_fold(init, acc);
        let root = N { val: 1, children: vec![N { val: 2, children: vec![] }] };

        // Honest base: user has a Treeish<N>, no seed-to-node step.
        // TreeishPipeline.lift().then_lift(Shared::explainer_lift()).run_from_node(&exec, &root).
        let trace = TreeishPipeline::new(
                graph::treeish(|n: &N| n.children.clone()),
                &fold_,
            )
            .lift()
            .then_lift(Shared::explainer_lift::<N, u64, u64>())
            .run_from_node(&dom::FUSED, &root);
        assert_eq!(trace.orig_result, 3);
    }
    // ANCHOR_END: explainer_usage

    // ANCHOR: parlazy_usage
    #[test]
    fn parlazy_usage() {
        use hylic_pipeline::prelude::{TreeishPipeline, PipelineExec};
        use hylic_parallel_lifts::{ParLazy, WorkPool, WorkPoolSpec};

        #[derive(Clone)]
        struct N { val: u64, children: Vec<N> }

        let init = |n: &N| n.val;
        let acc = |h: &mut u64, c: &u64| *h += c;
        let fold_ = dom::simple_fold(init, acc);
        let root = N { val: 1, children: vec![N { val: 2, children: vec![] }] };

        WorkPool::with(WorkPoolSpec::threads(2), |pool| {
            let parlazy = ParLazy::new(pool);
            // Compose ParLazy via then_lift; run_from_node to
            // get the lazy result, then evaluate it in parallel.
            let lazy = TreeishPipeline::new(
                    graph::treeish(|n: &N| n.children.clone()),
                    &fold_,
                )
                .lift()
                .then_lift(parlazy.clone())
                .run_from_node(&dom::FUSED, &root);
            let r = parlazy.eval(lazy);
            assert_eq!(r, 3);
        });
    }
    // ANCHOR_END: parlazy_usage

    // ANCHOR: pareager_usage
    #[test]
    fn pareager_usage() {
        use hylic_parallel_lifts::{ParEager, EagerSpec, WorkPool, WorkPoolSpec};

        #[derive(Clone)]
        struct N { val: u64, children: Vec<N> }

        let graph = graph::treeish(|n: &N| n.children.clone());
        let init = |n: &N| n.val;
        let acc = |h: &mut u64, c: &u64| *h += c;
        let fold = dom::simple_fold(init, acc);
        let root = N { val: 1, children: vec![N { val: 2, children: vec![] }] };

        WorkPool::with(WorkPoolSpec::threads(2), |pool| {
            let r = ParEager::lift(pool, EagerSpec::default_for(3)).run(&dom::FUSED, &fold, &graph, &root);
            assert_eq!(r, 3);
        });
    }
    // ANCHOR_END: pareager_usage

    // ── guides/execution.md examples ───────────────────

    // ANCHOR: domain_switching
    #[test]
    fn domain_switching() {

        #[derive(Clone)]
        struct N { val: u64, children: Vec<N> }

        let init = |n: &N| n.val;
        let acc = |h: &mut u64, c: &u64| *h += c;
        let fin = |h: &u64| *h;
        fn children(n: &N, cb: &mut dyn FnMut(&N)) {
            for c in &n.children { cb(c); }
        }

        let root = N { val: 1, children: vec![N { val: 2, children: vec![] }] };

        // Shared domain (standard):
        let fold = dom::fold(init, acc, fin);
        let graph = graph::treeish_visit(children);
        let r1 = dom::FUSED.run(&fold, &graph, &root);

        // Local domain (Rc, lighter):
        let fold = hylic::domain::local::fold(init, acc, fin);
        let graph = graph::treeish_visit(children);
        let r2 = hylic::domain::local::FUSED.run(&fold, &graph, &root);

        // Owned domain (Box, zero refcount):
        let fold = hylic::domain::owned::fold(init, acc, fin);
        let graph = graph::treeish_visit(children);
        let r3 = hylic::domain::owned::FUSED.run(&fold, &graph, &root);

        assert_eq!(r1, r2);
        assert_eq!(r2, r3);
    }
    // ANCHOR_END: domain_switching

    // ANCHOR: runtime_dispatch
    #[test]
    fn runtime_dispatch() {
        use hylic::cata::exec::funnel;

        #[derive(Clone)]
        struct N { val: u64, children: Vec<N> }

        let graph = graph::treeish(|n: &N| n.children.clone());
        let init = |n: &N| n.val;
        let acc = |h: &mut u64, c: &u64| *h += c;
        let fold = dom::simple_fold(init, acc);
        let root = N { val: 1, children: vec![N { val: 2, children: vec![] }] };

        // Same .run() for both — uniform interface
        let r1 = dom::FUSED.run(&fold, &graph, &root);
        let r2 = dom::exec(funnel::Spec::default(4)).run(&fold, &graph, &root);
        assert_eq!(r1, r2);
        assert_eq!(r1, 3);
    }
    // ANCHOR_END: runtime_dispatch

    // ── guides/graph.md examples ───────────────────────

    // ANCHOR: treeish_constructors
    #[test]
    fn treeish_constructors() {

        #[derive(Clone)]
        struct Node { value: u64, children: Vec<Node> }

        let root = Node { value: 1, children: vec![Node { value: 2, children: vec![] }] };

        // Callback-based (zero allocation per visit):
        let g1 = graph::treeish_visit(|n: &Node, cb: &mut dyn FnMut(&Node)| {
            for child in &n.children { cb(child); }
        });

        // Vec-returning (allocates per visit):
        let g2 = graph::treeish(|n: &Node| n.children.clone());

        // Slice accessor (borrows, zero allocation):
        let g3 = graph::treeish_from(|n: &Node| n.children.as_slice());

        assert_eq!(g1.apply(&root).len(), 1);
        assert_eq!(g2.apply(&root).len(), 1);
        assert_eq!(g3.apply(&root).len(), 1);

        // Flat data — nodes are indices, children from adjacency list:
        let adj: Vec<Vec<usize>> = vec![vec![1, 2], vec![], vec![]];
        let g4 = graph::treeish_visit(move |n: &usize, cb: &mut dyn FnMut(&usize)| {
            for &c in &adj[*n] { cb(&c); }
        });
        assert_eq!(g4.apply(&0).len(), 2);
    }
    // ANCHOR_END: treeish_constructors

    // ANCHOR: graph_filter
    #[test]
    fn graph_filter() {

        #[derive(Clone)]
        struct Node { value: u64, children: Vec<Node> }

        let graph = graph::treeish(|n: &Node| n.children.clone());
        let init = |n: &Node| n.value;
        let acc = |h: &mut u64, c: &u64| *h += c;
        let fold = dom::simple_fold(init, acc);

        let root = Node { value: 1, children: vec![
            Node { value: 10, children: vec![] },
            Node { value: 2, children: vec![] },
        ]};

        // Only visit children with value > 5
        let pruned = graph.filter(|child: &Node| child.value > 5);
        let result = dom::FUSED.run(&fold, &pruned, &root);
        assert_eq!(result, 11); // 1 + 10 (skipped 2)
    }
    // ANCHOR_END: graph_filter

    // ANCHOR: memoize_example
    #[test]
    fn memoize_example() {
        use hylic::prelude::memoize_treeish;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let call_count = Arc::new(AtomicUsize::new(0));
        let cc = call_count.clone();

        let graph = graph::treeish(move |n: &u64| -> Vec<u64> {
            cc.fetch_add(1, Ordering::Relaxed);
            if *n == 0 { vec![] } else { vec![n - 1] }
        });

        let init = |n: &u64| *n;
        let acc = |h: &mut u64, c: &u64| *h += c;
        let fold = dom::simple_fold(init, acc);

        let cached = memoize_treeish(&graph);
        let _ = dom::FUSED.run(&fold, &cached, &3u64);
        let first_count = call_count.load(Ordering::Relaxed);

        // Second run uses cache
        let _ = dom::FUSED.run(&fold, &cached, &3u64);
        let second_count = call_count.load(Ordering::Relaxed);
        assert_eq!(first_count, second_count); // no new calls
    }
    // ANCHOR_END: memoize_example

    // ── guides/fold.md examples ────────────────────────

    // ANCHOR: named_closures_pattern
    #[test]
    fn named_closures_pattern() {

        #[derive(Clone)]
        struct N { val: u64, children: Vec<N> }

        // Named closures — reusable across domains
        let init = |n: &N| n.val;
        let acc = |h: &mut u64, c: &u64| *h += c;
        let fin = |h: &u64| *h;
        fn children(n: &N, cb: &mut dyn FnMut(&N)) {
            for c in &n.children { cb(c); }
        }

        // Shared domain
        let fold = dom::fold(init, acc, fin);
        let graph = graph::treeish_visit(children);
        let root = N { val: 1, children: vec![N { val: 2, children: vec![] }] };

        assert_eq!(dom::FUSED.run(&fold, &graph, &root), 3);
    }
    // ANCHOR_END: named_closures_pattern

    // ANCHOR: fold_zipmap
    #[test]
    fn fold_zipmap() {

        #[derive(Clone)]
        struct N { val: u64, children: Vec<N> }

        let graph = graph::treeish(|n: &N| n.children.clone());
        let init = |n: &N| n.val;
        let acc = |h: &mut u64, c: &u64| *h += c;
        let fold = dom::simple_fold(init, acc);

        let with_flag = fold.zipmap(|r: &u64| *r > 5);
        let root = N { val: 1, children: vec![
            N { val: 3, children: vec![] },
            N { val: 4, children: vec![] },
        ]};
        let (total, over_five) = dom::FUSED.run(&with_flag, &graph, &root);
        assert_eq!(total, 8);
        assert!(over_five);
    }
    // ANCHOR_END: fold_zipmap

    // ANCHOR: fold_contramap
    #[test]
    fn fold_contramap() {

        #[derive(Clone)]
        struct N { val: u64, children: Vec<N> }

        let init = |n: &N| n.val;
        let acc = |h: &mut u64, c: &u64| *h += c;
        let fold = dom::simple_fold(init, acc);

        // Change node type: String → N
        let by_name = fold.contramap_n(|s: &String| N { val: s.len() as u64, children: vec![] });
        let graph = graph::treeish_visit(|_: &String, _cb: &mut dyn FnMut(&String)| {});

        let result = dom::FUSED.run(&by_name, &graph, &"hello".to_string());
        assert_eq!(result, 5);
    }
    // ANCHOR_END: fold_contramap

    // ── intro.md ───────────────────────────────────────

    // ANCHOR: intro_dir_example
    #[test]
    fn intro_dir_example() {
        use hylic::cata::exec::funnel;

        #[derive(Clone)]
        struct Dir { name: String, size: u64, children: Vec<Dir> }

        let graph = graph::treeish(|d: &Dir| d.children.clone());
        let fold = dom::simple_fold(
            |d: &Dir| d.size,
            |heap: &mut u64, child: &u64| *heap += child,
        );

        let tree = Dir {
            name: "project".into(), size: 10,
            children: vec![
                Dir { name: "src".into(), size: 200, children: vec![] },
                Dir { name: "docs".into(), size: 50, children: vec![] },
            ],
        };

        // Sequential:
        let total = dom::FUSED.run(&fold, &graph, &tree);
        assert_eq!(total, 260);

        // Parallel — same fold, same graph:
        let total = dom::exec(funnel::Spec::default(4)).run(&fold, &graph, &tree);
        assert_eq!(total, 260);
    }
    // ANCHOR_END: intro_dir_example

    // ANCHOR: intro_flat_example
    #[test]
    fn intro_flat_example() {
        // Flat adjacency list — nodes are indices, children are looked up
        let children: Vec<Vec<usize>> = vec![
            vec![1, 2],  // node 0 → children 1, 2
            vec![],      // node 1 → leaf
            vec![],      // node 2 → leaf
        ];
        let graph = graph::treeish_visit(move |n: &usize, cb: &mut dyn FnMut(&usize)| {
            for &c in &children[*n] { cb(&c); }
        });
        let fold = dom::simple_fold(|n: &usize| *n as u64, |h: &mut u64, c: &u64| *h += c);

        let total = dom::FUSED.run(&fold, &graph, &0);
        assert_eq!(total, 3); // 0 + 1 + 2
    }
    // ANCHOR_END: intro_flat_example

    // ── quickstart.md ─────────────────────────────────

    // ANCHOR: quickstart_funnel
    #[test]
    fn quickstart_funnel() {
        use hylic::cata::exec::funnel;

        #[derive(Clone)]
        struct Dir { name: String, size: u64, children: Vec<Dir> }

        let graph = graph::treeish(|d: &Dir| d.children.clone());
        let fold = dom::simple_fold(
            |d: &Dir| d.size,
            |heap: &mut u64, child: &u64| *heap += child,
        );

        let tree = Dir {
            name: "root".into(), size: 10,
            children: vec![
                Dir { name: "a".into(), size: 5, children: vec![] },
                Dir { name: "b".into(), size: 3, children: vec![] },
            ],
        };

        let total = dom::exec(funnel::Spec::default(4)).run(&fold, &graph, &tree);
        assert_eq!(total, 18);
    }
    // ANCHOR_END: quickstart_funnel

    // ANCHOR: quickstart_session
    #[test]
    fn quickstart_session() {
        use hylic::cata::exec::funnel;

        #[derive(Clone)]
        struct Dir { name: String, size: u64, children: Vec<Dir> }

        let graph = graph::treeish(|d: &Dir| d.children.clone());
        let fold = dom::simple_fold(
            |d: &Dir| d.size,
            |heap: &mut u64, child: &u64| *heap += child,
        );

        let tree = Dir {
            name: "root".into(), size: 10,
            children: vec![
                Dir { name: "a".into(), size: 5, children: vec![] },
            ],
        };

        dom::exec(funnel::Spec::default(4)).session(|s| {
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
        use hylic_pipeline::prelude::{SeedPipeline, PipelineExecSeed};
        use std::collections::HashMap;

        // The "registry" — flat data, not a tree
        let mut modules: HashMap<String, Vec<String>> = HashMap::new();
        modules.insert("app".into(), vec!["db".into(), "auth".into()]);
        modules.insert("db".into(), vec![]);
        modules.insert("auth".into(), vec!["db".into()]);

        // The seed edge function: given a module name, produce its dependency seeds
        let reg = modules.clone();
        let seeds_from_node = graph::edgy_visit(move |name: &String, cb: &mut dyn FnMut(&String)| {
            if let Some(deps) = reg.get(name) {
                for dep in deps { cb(dep); }
            }
        });

        // The fold: collect all reachable names
        let fold = dom::fold(
            |name: &String| vec![name.clone()],
            |heap: &mut Vec<String>, child: &Vec<String>| heap.extend(child.iter().cloned()),
            |heap: &Vec<String>| heap.clone(),
        );

        let pipeline = SeedPipeline::new(
            |seed: &String| seed.clone(),
            seeds_from_node,
            &fold,
        );

        let result = pipeline.run_from_slice(
            &dom::FUSED,
            &["app".to_string()],
            Vec::<String>::new(),
        );
        assert!(result.contains(&"app".to_string()));
        assert!(result.contains(&"auth".to_string()));
    }
    // ANCHOR_END: seed_pipeline_example

    // ANCHOR: seed_pipeline_parallel
    #[test]
    fn seed_pipeline_parallel() {
        use hylic::cata::exec::funnel;
        use hylic_pipeline::prelude::{SeedPipeline, PipelineExecSeed};
        use std::collections::HashMap;

        let mut modules: HashMap<String, Vec<String>> = HashMap::new();
        modules.insert("app".into(), vec!["db".into(), "auth".into()]);
        modules.insert("db".into(), vec![]);
        modules.insert("auth".into(), vec!["db".into()]);

        let reg = modules.clone();
        let seeds_from_node = graph::edgy_visit(move |name: &String, cb: &mut dyn FnMut(&String)| {
            if let Some(deps) = reg.get(name) { for dep in deps { cb(dep); } }
        });

        let fold = dom::fold(
            |name: &String| vec![name.clone()],
            |heap: &mut Vec<String>, child: &Vec<String>| heap.extend(child.iter().cloned()),
            |heap: &Vec<String>| heap.clone(),
        );

        let pipeline = SeedPipeline::new(
            |seed: &String| seed.clone(),
            seeds_from_node,
            &fold,
        );

        let result = pipeline.run_from_slice(
            &dom::exec(funnel::Spec::default(4)),
            &["app".to_string()],
            Vec::<String>::new(),
        );
        assert!(result.contains(&"app".to_string()));
    }
    // ANCHOR_END: seed_pipeline_parallel
}

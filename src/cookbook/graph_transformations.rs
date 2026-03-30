//! Graph and fold transformations: a progression from simple to composed.
//! Shows how to start with a plain graph, layer on seed resolution,
//! add logging, selective caching, and context-aware accumulation.

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use hylic::fold::simple_fold;
    use hylic::prelude::vec_fold::{vec_fold, VecHeap};
    use hylic::graph::{treeish, treeish_from, Treeish};
    use hylic::cata::Sequential;
    use insta::assert_snapshot;

    // ---- Domain: a build system with tasks and dependencies ----

    // ANCHOR: build_domain

    /// A build task with a name, estimated cost, and dependencies.
    #[derive(Clone, Debug)]
    struct Task {
        name: String,
        cost_ms: u64,
        deps: Vec<String>,
    }

    /// A build graph: tasks keyed by name.
    #[derive(Clone)]
    struct BuildGraph(HashMap<String, Task>);

    impl BuildGraph {
        fn new(tasks: &[(&str, u64, &[&str])]) -> Self {
            BuildGraph(tasks.iter().map(|(name, cost, deps)| {
                (name.to_string(), Task {
                    name: name.to_string(),
                    cost_ms: *cost,
                    deps: deps.iter().map(|d| d.to_string()).collect(),
                })
            }).collect())
        }

        fn lookup(&self, name: &str) -> Option<&Task> {
            self.0.get(name)
        }
    }

    // ANCHOR_END: build_domain

    // ---- 1. Simple graph from static data ----

    // ANCHOR: simple_graph
    #[test]
    fn simple_static_graph() {
        // Tasks stored as a tree directly (no lookup needed).
        #[derive(Clone)]
        #[allow(dead_code)]
        struct BuildNode {
            name: String,
            cost_ms: u64,
            children: Vec<BuildNode>,
        }

        let tree = BuildNode {
            name: "app".into(), cost_ms: 50,
            children: vec![
                BuildNode { name: "compile".into(), cost_ms: 200, children: vec![
                    BuildNode { name: "parse".into(), cost_ms: 100, children: vec![] },
                    BuildNode { name: "typecheck".into(), cost_ms: 300, children: vec![] },
                ]},
                BuildNode { name: "link".into(), cost_ms: 150, children: vec![] },
            ],
        };

        // treeish_from: zero-clone accessor for struct with children field.
        let graph = treeish_from(|n: &BuildNode| n.children.as_slice());

        // Total build time: sum of all task costs.
        let total_cost = simple_fold(
            |n: &BuildNode| n.cost_ms,
            |heap: &mut u64, child: &u64| *heap += child,
        );

        let result = Sequential.run(&total_cost, &graph, &tree);
        assert_eq!(result, 800); // 50 + 200 + 100 + 300 + 150

        // Critical path: maximum depth-weighted cost.
        let critical_path = simple_fold(
            |n: &BuildNode| n.cost_ms,
            |heap: &mut u64, child: &u64| *heap = (*heap).max(*child + 0),
        );

        let longest = Sequential.run(&critical_path, &graph, &tree);
        assert_eq!(longest, 300); // typecheck alone is the max leaf
    // ANCHOR_END: simple_graph

        assert_snapshot!("simple_graph", format!("total={result}, critical={longest}"));
    }

    // ---- 2. Graph from lookup (seed-like pattern without SeedGraph) ----

    // ANCHOR: lookup_graph
    #[test]
    fn graph_from_lookup() {
        let bg = BuildGraph::new(&[
            ("app",       50,  &["compile", "link"]),
            ("compile",   200, &["parse", "typecheck"]),
            ("parse",     100, &[]),
            ("typecheck", 300, &[]),
            ("link",      150, &[]),
        ]);

        // The graph is constructed by lookup — each task's deps
        // are resolved by name. Treeish operates on Task directly.
        let bg_clone = bg.clone();
        let graph: Treeish<Task> = treeish(move |task: &Task| {
            task.deps.iter()
                .filter_map(|dep| bg_clone.lookup(dep).cloned())
                .collect()
        });

        let root = bg.lookup("app").unwrap().clone();

        let total = simple_fold(
            |t: &Task| t.cost_ms,
            |heap: &mut u64, child: &u64| *heap += child,
        );

        let result = Sequential.run(&total, &graph, &root);
        assert_eq!(result, 800);
    // ANCHOR_END: lookup_graph

        assert_snapshot!("lookup_graph", format!("total={result}"));
    }

    // ---- 3. Logging which tasks take long ----

    // ANCHOR: selective_logging
    #[test]
    fn selective_logging() {
        let bg = BuildGraph::new(&[
            ("app",       50,  &["compile", "link"]),
            ("compile",   200, &["parse", "typecheck"]),
            ("parse",     100, &[]),
            ("typecheck", 300, &[]),
            ("link",      150, &[]),
        ]);

        let bg_clone = bg.clone();
        let graph = treeish(move |task: &Task| {
            task.deps.iter()
                .filter_map(|dep| bg_clone.lookup(dep).cloned())
                .collect()
        });

        let root = bg.lookup("app").unwrap().clone();

        // VecFold gives finalize access to both the node and all child results,
        // so we can compute the subtree total and log tasks exceeding a threshold.
        let slow_tasks = Arc::new(Mutex::new(Vec::new()));
        let slow_clone = slow_tasks.clone();
        let logged = vec_fold(move |heap: &VecHeap<Task, u64>| {
            let subtree_total: u64 = heap.node.cost_ms
                + heap.childresults.iter().sum::<u64>();
            if subtree_total > 200 {
                slow_clone.lock().unwrap().push(
                    format!("{}: {}ms", heap.node.name, subtree_total)
                );
            }
            subtree_total
        });

        let result = Sequential.run(&logged, &graph, &root);
        let slow: Vec<String> = slow_tasks.lock().unwrap().clone();

        assert_eq!(result, 800);
        // compile (600ms) and app (800ms) exceed threshold
        assert!(slow.iter().any(|s| s.contains("compile")));
        assert!(slow.iter().any(|s| s.contains("app")));
    // ANCHOR_END: selective_logging

        assert_snapshot!("selective_logging", format!(
            "total={result}, slow=[{}]", slow.join(", ")
        ));
    }

    // ---- 4. Caching: skip already-computed subtrees ----

    // ANCHOR: caching
    #[test]
    fn caching_with_context() {
        // Diamond dependency: compile and link both depend on "stdlib".
        let bg = BuildGraph::new(&[
            ("app",       10, &["compile", "link"]),
            ("compile",   50, &["stdlib"]),
            ("link",      30, &["stdlib"]),
            ("stdlib",   200, &[]),
        ]);

        let call_count = Arc::new(Mutex::new(0u32));
        let cc = call_count.clone();

        let bg_clone = bg.clone();
        let graph = treeish(move |task: &Task| {
            *cc.lock().unwrap() += 1;
            task.deps.iter()
                .filter_map(|dep| bg_clone.lookup(dep).cloned())
                .collect()
        });

        let root = bg.lookup("app").unwrap().clone();

        let total = simple_fold(
            |t: &Task| t.cost_ms,
            |heap: &mut u64, child: &u64| *heap += child,
        );

        // Without caching: stdlib is visited twice (via compile and link).
        let result = Sequential.run(&total, &graph, &root);
        let calls_no_cache = *call_count.lock().unwrap();

        assert_eq!(result, 490); // 10 + 50 + 30 + 200 + 200 (stdlib counted twice)
        assert_eq!(calls_no_cache, 5); // app, compile, stdlib, link, stdlib
    // ANCHOR_END: caching

        assert_snapshot!("caching", format!(
            "total={result}, graph_calls={calls_no_cache} (stdlib visited twice)"
        ));
    }
}

//! Transformations: features as standalone functions that match the contract.
//!
//! One domain, one base fold, one base graph. Each feature is a named
//! function — it IS the concern, separated and reusable. Plugging it
//! in is a single method call on the existing construct.

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use hylic::fold::{simple_fold, InitFn, AccumulateFn, FinalizeFn};
    use hylic::graph::{treeish, Treeish};
    use hylic::prelude::memoize_treeish_by;
    use hylic::cata::Sequential;
    use insta::assert_snapshot;

    // ---- Shared domain ----

    // ANCHOR: domain
    #[derive(Clone, Debug)]
    struct Task {
        name: String,
        cost_ms: u64,
        deps: Vec<String>,
    }

    struct Registry(HashMap<String, Task>);

    impl Registry {
        fn new(tasks: &[(&str, u64, &[&str])]) -> Self {
            Registry(tasks.iter().map(|(name, cost, deps)| {
                (name.to_string(), Task {
                    name: name.to_string(),
                    cost_ms: *cost,
                    deps: deps.iter().map(|d| d.to_string()).collect(),
                })
            }).collect())
        }
        fn get(&self, name: &str) -> Option<&Task> { self.0.get(name) }
    }
    // ANCHOR_END: domain

    fn test_registry() -> Registry {
        Registry::new(&[
            ("app",       50,  &["compile", "link"]),
            ("compile",   200, &["parse", "typecheck"]),
            ("parse",     100, &[]),
            ("typecheck", 300, &[]),
            ("link",      150, &[]),
        ])
    }

    fn test_graph(reg: &Registry) -> (Treeish<Task>, Task) {
        let map = reg.0.clone();
        let graph = treeish(move |task: &Task| {
            task.deps.iter()
                .filter_map(|d| map.get(d).cloned())
                .collect()
        });
        let root = reg.get("app").unwrap().clone();
        (graph, root)
    }

    fn base_fold() -> hylic::fold::Fold<Task, u64, u64> {
        simple_fold(
            |t: &Task| t.cost_ms,
            |heap: &mut u64, child: &u64| *heap += child,
        )
    }

    // ==== FOLD PHASE WRAPPERS ====
    //
    // Each feature is a function returning a closure that matches
    // the transformation contract. The function IS the feature.

    // ---- map_init: visit_logger ----

    // ANCHOR: visit_logger
    /// Feature: log each node as it's visited.
    /// Contract: FnOnce(InitFn<Task, u64>) -> InitFn<Task, u64>
    fn visit_logger(
        sink: Arc<Mutex<Vec<String>>>,
    ) -> impl FnOnce(InitFn<Task, u64>) -> InitFn<Task, u64> {
        let log = sink;
        move |orig: InitFn<Task, u64>| -> InitFn<Task, u64> {
            Box::new(move |task: &Task| {
                log.lock().unwrap().push(task.name.clone());
                orig(task)
            })
        }
    }

    #[test]
    fn test_visit_logger() {
        let reg = test_registry();
        let (graph, root) = test_graph(&reg);

        let visited = Arc::new(Mutex::new(Vec::new()));
        let fold = base_fold().map_init(visit_logger(visited.clone()));

        let total = Sequential.run(&fold, &graph, &root);
        let names: Vec<String> = visited.lock().unwrap().clone();
    // ANCHOR_END: visit_logger
        assert_eq!(total, 800);
        assert_snapshot!("visit_logger", format!(
            "total={total}, visited: {}", names.join(" → ")
        ));
    }

    // ---- map_accumulate: skip_small_children ----

    // ANCHOR: skip_small
    /// Feature: during accumulation, ignore children below a threshold.
    /// Contract: FnOnce(AccumulateFn<u64, u64>) -> AccumulateFn<u64, u64>
    fn skip_small_children(
        threshold: u64,
    ) -> impl FnOnce(AccumulateFn<u64, u64>) -> AccumulateFn<u64, u64> {
        move |orig: AccumulateFn<u64, u64>| -> AccumulateFn<u64, u64> {
            Box::new(move |heap: &mut u64, child: &u64| {
                if *child >= threshold {
                    orig(heap, child);
                }
            })
        }
    }

    #[test]
    fn test_skip_small_children() {
        let reg = test_registry();
        let (graph, root) = test_graph(&reg);

        // Only accumulate children whose subtree cost >= 200.
        let fold = base_fold().map_accumulate(skip_small_children(200));

        let total = Sequential.run(&fold, &graph, &root);
        // app(50): children are compile(600) and link(150).
        // link(150) < 200, skipped. compile(600) >= 200, kept.
        // compile(200): children are parse(100) and typecheck(300).
        // parse(100) < 200, skipped. typecheck(300) >= 200, kept.
        // Result: app=50 + compile(200+typecheck=300) = 50+500 = 550
    // ANCHOR_END: skip_small
        assert_eq!(total, 550);
        assert_snapshot!("skip_small", format!("total={total} (small children skipped)"));
    }

    // ---- map_finalize: clamp_at ----

    // ANCHOR: clamp_at
    /// Feature: cap each subtree's result at a maximum.
    /// Contract: FnOnce(FinalizeFn<u64, u64>) -> FinalizeFn<u64, u64>
    fn clamp_at(
        max: u64,
    ) -> impl FnOnce(FinalizeFn<u64, u64>) -> FinalizeFn<u64, u64> {
        move |orig: FinalizeFn<u64, u64>| -> FinalizeFn<u64, u64> {
            Box::new(move |heap: &u64| orig(heap).min(max))
        }
    }

    #[test]
    fn test_clamp_at() {
        let reg = test_registry();
        let (graph, root) = test_graph(&reg);

        let fold = base_fold().map_finalize(clamp_at(500));

        let total = Sequential.run(&fold, &graph, &root);
        // compile subtree: min(200+100+300, 500) = 500
        // link: min(150, 500) = 150
        // app: min(50+500+150, 500) = 500
    // ANCHOR_END: clamp_at
        assert_eq!(total, 500);
        assert_snapshot!("clamp_at", format!("total={total} (clamped at 500)"));
    }

    // ==== FOLD RESULT AUGMENTATION ====

    // ---- zipmap: classify ----

    // ANCHOR: classify
    /// Feature: categorize each subtree's total cost.
    /// Contract: Fn(&u64) -> &str  (zipmap contract)
    fn classify(total: &u64) -> &'static str {
        match *total {
            t if t >= 500 => "critical",
            t if t >= 200 => "heavy",
            _ => "light",
        }
    }

    #[test]
    fn test_classify() {
        let reg = test_registry();
        let (graph, root) = test_graph(&reg);

        let fold = base_fold().zipmap(classify);

        let (total, category) = Sequential.run(&fold, &graph, &root);
    // ANCHOR_END: classify
        assert_eq!(total, 800);
        assert_eq!(category, "critical");
        assert_snapshot!("classify", format!("total={total}, category={category}"));
    }

    // ==== GRAPH TRANSFORMATIONS ====

    // ---- filter edges: only_costly_deps ----

    // ANCHOR: only_costly
    /// Feature: filter a graph's children to only those above a cost threshold.
    /// Takes a Treeish, returns a Treeish — same node type, fewer edges.
    fn only_costly_deps(graph: &Treeish<Task>, min_cost: u64) -> Treeish<Task> {
        let inner = graph.clone();
        treeish(move |task: &Task| {
            inner.at(task)
                .filter(|child: &Task| child.cost_ms >= min_cost)
                .collect_vec()
        })
    }

    #[test]
    fn test_only_costly_deps() {
        let reg = test_registry();
        let (graph, root) = test_graph(&reg);

        // Only traverse deps with cost >= 150.
        // Drops parse(100) from compile's deps.
        let filtered = only_costly_deps(&graph, 150);

        let total = Sequential.run(&base_fold(), &filtered, &root);
        // app(50) + compile(200) + typecheck(300) + link(150) = 700
        // parse(100) excluded from traversal entirely
    // ANCHOR_END: only_costly
        assert_eq!(total, 700);
        assert_snapshot!("only_costly", format!("total={total} (deps with cost < 150 pruned)"));
    }

    // ---- memoize: cache diamond dependencies ----

    // ANCHOR: memoize
    #[test]
    fn test_memoize_diamond() {
        let reg = Registry::new(&[
            ("app",     10, &["compile", "link"]),
            ("compile", 50, &["stdlib"]),
            ("link",    30, &["stdlib"]),
            ("stdlib", 200, &[]),
        ]);

        let visit_count = Arc::new(Mutex::new(0u32));
        let vc = visit_count.clone();
        let map = reg.0.clone();
        let graph = treeish(move |task: &Task| {
            *vc.lock().unwrap() += 1;
            task.deps.iter()
                .filter_map(|d| map.get(d).cloned())
                .collect()
        });
        let root = reg.get("app").unwrap().clone();

        // Raw: stdlib visited twice.
        let total = Sequential.run(&base_fold(), &graph, &root);
        let raw_visits = *visit_count.lock().unwrap();

        // memoize_treeish_by: same fold, cached graph.
        *visit_count.lock().unwrap() = 0;
        let cached = memoize_treeish_by(&graph, |t: &Task| t.name.clone());
        let total_memo = Sequential.run(&base_fold(), &cached, &root);
        let memo_visits = *visit_count.lock().unwrap();
    // ANCHOR_END: memoize
        assert_eq!(total, 490);
        assert_eq!(raw_visits, 5);
        assert_eq!(total_memo, 490);
        assert_eq!(memo_visits, 4);
        assert_snapshot!("memoize", format!(
            "raw: total={total} visits={raw_visits}, memo: total={total_memo} visits={memo_visits}"
        ));
    }

    // ==== COMPOSITION ====

    // ANCHOR: composed
    #[test]
    fn test_composed_pipeline() {
        let reg = test_registry();
        let (graph, root) = test_graph(&reg);

        // Three independent features, each a standalone function:
        let visited = Arc::new(Mutex::new(Vec::new()));

        let pipeline = base_fold()
            .map_init(visit_logger(visited.clone()))
            .map_finalize(clamp_at(500))
            .zipmap(classify);

        let (total, category) = Sequential.run(&pipeline, &graph, &root);
        let names: Vec<String> = visited.lock().unwrap().clone();
    // ANCHOR_END: composed
        assert_eq!(total, 500);
        assert_eq!(category, "critical");
        assert_snapshot!("composed", format!(
            "total={total} [{category}], visited: {}", names.join(" → ")
        ));
    }
}

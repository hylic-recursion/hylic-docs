//! Transformations: features as standalone functions that match the contract.
//!
//! One domain, one base fold, one base graph. Each feature is a named
//! function — it IS the concern, separated and reusable. Plugging it
//! in is a single method call on the existing construct.

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use hylic::fold::{simple_fold, Fold, InitFn, AccumulateFn, FinalizeFn};
    use hylic::graph::{treeish, Treeish};
    use hylic::prelude::memoize_treeish_by;
    use hylic::cata::exec::{self, Executor};
    use insta::assert_snapshot;

    // ── Domain ──────────────────────────────────────────────

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

    // ── Shared setup ────────────────────────────────────────

    fn setup() -> (Treeish<Task>, Task) {
        let reg = Registry::new(&[
            ("app",       50,  &["compile", "link"]),
            ("compile",   200, &["parse", "typecheck"]),
            ("parse",     100, &[]),
            ("typecheck", 300, &[]),
            ("link",      150, &[]),
        ]);
        let map = reg.0.clone();
        let graph = treeish(move |task: &Task| {
            task.deps.iter().filter_map(|d| map.get(d).cloned()).collect()
        });
        let root = reg.get("app").unwrap().clone();
        (graph, root)
    }

    fn base_fold() -> Fold<Task, u64, u64> {
        let init = |t: &Task| t.cost_ms;
        let acc = |heap: &mut u64, child: &u64| *heap += child;
        simple_fold(init, acc)
    }

    // ── Fold phase wrappers ─────────────────────────────────
    //
    // Each is a standalone function returning a closure that
    // matches the transformation contract.

    /// Hooks into init: called once per node, before any children are processed.
    /// Receives the original init and wraps it — original behavior is preserved,
    /// the side effect (logging) is layered on top.
    fn visit_logger(sink: Arc<Mutex<Vec<String>>>)
        -> impl FnOnce(InitFn<Task, u64>) -> InitFn<Task, u64>
    {
        move |orig: InitFn<Task, u64>| -> InitFn<Task, u64> {
            Box::new(move |task: &Task| {
                sink.lock().unwrap().push(task.name.clone());
                orig(task) // original init still runs — we add, not replace
            })
        }
    }

    /// Hooks into accumulate: called once per child result as it's folded in.
    /// By conditionally calling orig, some children's results are simply
    /// never folded — the parent doesn't see them.
    fn skip_small_children(threshold: u64)
        -> impl FnOnce(AccumulateFn<u64, u64>) -> AccumulateFn<u64, u64>
    {
        move |orig: AccumulateFn<u64, u64>| -> AccumulateFn<u64, u64> {
            Box::new(move |heap: &mut u64, child: &u64| {
                if *child >= threshold { orig(heap, child); }
            })
        }
    }

    /// Hooks into finalize: called once per node, after all children
    /// are accumulated. The heap holds the fully-accumulated value;
    /// the clamp applies to the result seen by this node's parent.
    fn clamp_at(max: u64)
        -> impl FnOnce(FinalizeFn<u64, u64>) -> FinalizeFn<u64, u64>
    {
        move |orig: FinalizeFn<u64, u64>| -> FinalizeFn<u64, u64> {
            Box::new(move |heap: &u64| orig(heap).min(max))
        }
    }

    /// zipmap contract: a plain Fn(&R) -> Extra. No wrapping needed —
    /// the function itself IS the feature. zipmap calls it per node,
    /// pairing the original result with the derived value: R → (R, Extra).
    fn classify(total: &u64) -> &'static str {
        match *total {
            t if t >= 500 => "critical",
            t if t >= 200 => "heavy",
            _ => "light",
        }
    }

    // ── Graph transformations ───────────────────────────────

    /// Graph-level transformation: wraps a Treeish, returns a Treeish.
    /// Same node type, fewer edges — the fold is completely unchanged.
    /// Uses Edgy::at() which returns a Visit (push-based iterator with
    /// filter/map/collect — zero-allocation unless collected).
    fn only_costly_deps(graph: &Treeish<Task>, min_cost: u64) -> Treeish<Task> {
        let inner = graph.clone();
        treeish(move |task: &Task| {
            inner.at(task)
                .filter(|child: &Task| child.cost_ms >= min_cost)
                .collect_vec()
        })
    }

    // ── Tests ───────────────────────────────────────────────

    #[test]
    fn test_visit_logger() {
        let (graph, root) = setup();
        let visited = Arc::new(Mutex::new(Vec::new()));
        let fold = base_fold().map_init(visit_logger(visited.clone()));

        let total = exec::FUSED.run(&fold, &graph, &root);
        let names: Vec<String> = visited.lock().unwrap().clone();
        assert_eq!(total, 800);
        assert_snapshot!("visit_logger", format!(
            "total={total}, visited: {}", names.join(" → ")
        ));
    }

    #[test]
    fn test_skip_small_children() {
        let (graph, root) = setup();
        let fold = base_fold().map_accumulate(skip_small_children(200));
        let total = exec::FUSED.run(&fold, &graph, &root);
        // app(50) + compile(200+typecheck 300) = 550; parse(100) and link(150) skipped
        assert_eq!(total, 550);
        assert_snapshot!("skip_small", format!("total={total} (small children skipped)"));
    }

    #[test]
    fn test_clamp_at() {
        let (graph, root) = setup();
        let fold = base_fold().map_finalize(clamp_at(500));
        let total = exec::FUSED.run(&fold, &graph, &root);
        // compile=min(600,500)=500, link=150, app=min(50+500+150,500)=500
        assert_eq!(total, 500);
        assert_snapshot!("clamp_at", format!("total={total} (clamped at 500)"));
    }

    #[test]
    fn test_classify() {
        let (graph, root) = setup();
        let (total, category) = exec::FUSED.run(&base_fold().zipmap(classify), &graph, &root);
        assert_eq!(total, 800);
        assert_eq!(category, "critical");
        assert_snapshot!("classify", format!("total={total}, category={category}"));
    }

    #[test]
    fn test_only_costly_deps() {
        let (graph, root) = setup();
        let filtered = only_costly_deps(&graph, 150);
        let total = exec::FUSED.run(&base_fold(), &filtered, &root);
        // parse(100) pruned: app(50)+compile(200)+typecheck(300)+link(150) = 700
        assert_eq!(total, 700);
        assert_snapshot!("only_costly", format!("total={total} (deps with cost < 150 pruned)"));
    }

    #[test]
    fn test_memoize_diamond() {
        let reg = Registry::new(&[
            ("app", 10, &["compile", "link"]),
            ("compile", 50, &["stdlib"]),
            ("link", 30, &["stdlib"]),
            ("stdlib", 200, &[]),
        ]);
        let visit_count = Arc::new(Mutex::new(0u32));
        let vc = visit_count.clone();
        let map = reg.0.clone();
        let graph = treeish(move |task: &Task| {
            *vc.lock().unwrap() += 1;
            task.deps.iter().filter_map(|d| map.get(d).cloned()).collect()
        });
        let root = reg.get("app").unwrap().clone();

        let total = exec::FUSED.run(&base_fold(), &graph, &root);
        let raw_visits = *visit_count.lock().unwrap();

        *visit_count.lock().unwrap() = 0;
        let cached = memoize_treeish_by(&graph, |t: &Task| t.name.clone());
        let total_memo = exec::FUSED.run(&base_fold(), &cached, &root);
        let memo_visits = *visit_count.lock().unwrap();

        assert_eq!((total, raw_visits), (490, 5));
        assert_eq!((total_memo, memo_visits), (490, 4));
        assert_snapshot!("memoize", format!(
            "raw: total={total} visits={raw_visits}, memo: total={total_memo} visits={memo_visits}"
        ));
    }

    #[test]
    fn test_composed_pipeline() {
        let (graph, root) = setup();
        let visited = Arc::new(Mutex::new(Vec::new()));
        let pipeline = base_fold()
            .map_init(visit_logger(visited.clone()))
            .map_finalize(clamp_at(500))
            .zipmap(classify);

        let (total, category) = exec::FUSED.run(&pipeline, &graph, &root);
        let names: Vec<String> = visited.lock().unwrap().clone();
        assert_eq!(total, 500);
        assert_eq!(category, "critical");
        assert_snapshot!("composed", format!(
            "total={total} [{category}], visited: {}", names.join(" → ")
        ));
    }
}

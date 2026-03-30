//! Fold and graph transformations: wrapping, layering, composing.
//!
//! All examples share one domain, one graph, one base fold.
//! Each transformation wraps an existing piece — the base is never modified.

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use hylic::fold::simple_fold;
    use hylic::graph::{treeish, Treeish};
    use hylic::prelude::memoize_treeish_by;
    use hylic::cata::Sequential;
    use insta::assert_snapshot;

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

    // ---- 1. The base: a fold that sums costs ----

    // ANCHOR: base
    #[test]
    fn base_fold() {
        let reg = test_registry();
        let (graph, root) = test_graph(&reg);

        // The base fold: sum all task costs bottom-up.
        let sum_cost = simple_fold(
            |t: &Task| t.cost_ms,
            |heap: &mut u64, child: &u64| *heap += child,
        );

        let total = Sequential.run(&sum_cost, &graph, &root);
        assert_eq!(total, 800);
    // ANCHOR_END: base
        assert_snapshot!("base", format!("total = {total}"));
    }

    // ---- 2. map_init: wrap init to add visit logging ----

    // ANCHOR: map_init
    #[test]
    fn wrap_with_logging() {
        let reg = test_registry();
        let (graph, root) = test_graph(&reg);

        let sum_cost = simple_fold(
            |t: &Task| t.cost_ms,
            |heap: &mut u64, child: &u64| *heap += child,
        );

        // Wrap init: log each node as it's visited.
        // sum_cost is unchanged — logged_sum is a new fold.
        let visited = Arc::new(Mutex::new(Vec::new()));
        let log = visited.clone();
        let logged_sum = sum_cost.map_init(move |orig_init| {
            Box::new(move |task: &Task| {
                log.lock().unwrap().push(task.name.clone());
                orig_init(task)
            })
        });

        let total = Sequential.run(&logged_sum, &graph, &root);
        let names: Vec<String> = visited.lock().unwrap().clone();
    // ANCHOR_END: map_init
        assert_eq!(total, 800);
        assert_snapshot!("map_init", format!(
            "total = {total}, visited: {}", names.join(" → ")
        ));
    }

    // ---- 3. map_finalize: post-process each node's result ----

    // ANCHOR: map_finalize
    #[test]
    fn clamp_results() {
        let reg = test_registry();
        let (graph, root) = test_graph(&reg);

        let sum_cost = simple_fold(
            |t: &Task| t.cost_ms,
            |heap: &mut u64, child: &u64| *heap += child,
        );

        // Wrap finalize: cap each subtree's total at 500ms.
        // Children still accumulate normally — the cap applies
        // after finalize, so it affects the result seen by parents.
        let capped = sum_cost.map_finalize(move |orig_finalize| {
            Box::new(move |heap: &u64| {
                let result = orig_finalize(heap);
                result.min(500)
            })
        });

        let total = Sequential.run(&capped, &graph, &root);
        // compile subtree: min(200+100+300, 500) = 500
        // link: min(150, 500) = 150
        // app: min(50+500+150, 500) = 500
        assert_eq!(total, 500);
    // ANCHOR_END: map_finalize
        assert_snapshot!("map_finalize", format!("capped total = {total}"));
    }

    // ---- 4. zipmap: augment result with per-node annotation ----

    // ANCHOR: zipmap
    #[test]
    fn classify_subtrees() {
        let reg = test_registry();
        let (graph, root) = test_graph(&reg);

        let sum_cost = simple_fold(
            |t: &Task| t.cost_ms,
            |heap: &mut u64, child: &u64| *heap += child,
        );

        // zipmap: derive a classification from each subtree's sum.
        // The accumulation is still sum — zipmap post-processes per node.
        let classified = sum_cost.zipmap(|total: &u64| {
            match *total {
                t if t >= 500 => "critical",
                t if t >= 200 => "heavy",
                _ => "light",
            }
        });

        let (total, category) = Sequential.run(&classified, &graph, &root);
        assert_eq!(total, 800);
        assert_eq!(category, "critical");
    // ANCHOR_END: zipmap
        assert_snapshot!("zipmap", format!(
            "total = {total}, root category = {category}"
        ));
    }

    // ---- 5. map: change the result type entirely ----

    // ANCHOR: map_result
    #[test]
    fn change_result_type() {
        let reg = test_registry();
        let (graph, root) = test_graph(&reg);

        let sum_cost = simple_fold(
            |t: &Task| t.cost_ms,
            |heap: &mut u64, child: &u64| *heap += child,
        );

        // map: transform u64 → String.
        // The backmapper lets children's String results flow back
        // through the original u64 accumulator.
        let as_report = sum_cost.map(
            |total: &u64| format!("{}ms", total),
            |s: &String| s.trim_end_matches("ms").parse::<u64>().unwrap(),
        );

        let report = Sequential.run(&as_report, &graph, &root);
        assert_eq!(report, "800ms");
    // ANCHOR_END: map_result
        assert_snapshot!("map_result", format!("report = {report}"));
    }

    // ---- 6. Graph: memoize for diamond dependencies ----

    // ANCHOR: memoize
    #[test]
    fn memoize_diamond() {
        // Diamond: compile and link both depend on stdlib.
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

        let sum_cost = simple_fold(
            |t: &Task| t.cost_ms,
            |heap: &mut u64, child: &u64| *heap += child,
        );

        // Without memoization: stdlib visited twice.
        let total = Sequential.run(&sum_cost, &graph, &root);
        let calls_raw = *visit_count.lock().unwrap();

        // Wrap the graph — same node type, same fold, just cached.
        *visit_count.lock().unwrap() = 0;
        let memo_graph = memoize_treeish_by(&graph, |t: &Task| t.name.clone());
        let total_memo = Sequential.run(&sum_cost, &memo_graph, &root);
        let calls_memo = *visit_count.lock().unwrap();
    // ANCHOR_END: memoize

        assert_eq!(total, 490); // stdlib counted twice: 200+200
        assert_eq!(calls_raw, 5); // app, compile, stdlib, link, stdlib
        assert_eq!(total_memo, 490);
        assert_eq!(calls_memo, 4); // stdlib cached on second visit

        assert_snapshot!("memoize", format!(
            "raw: total={total} visits={calls_raw}, memo: total={total_memo} visits={calls_memo}"
        ));
    }

    // ---- 7. Composition: stack multiple fold transforms ----

    // ANCHOR: composed
    #[test]
    fn stacked_transforms() {
        let reg = test_registry();
        let (graph, root) = test_graph(&reg);

        let sum_cost = simple_fold(
            |t: &Task| t.cost_ms,
            |heap: &mut u64, child: &u64| *heap += child,
        );

        // Stack: log visits, then classify the result.
        // Each transform wraps the previous — no rewriting.
        let visited = Arc::new(Mutex::new(Vec::new()));
        let log = visited.clone();
        let pipeline = sum_cost
            .map_init(move |orig| {
                Box::new(move |t: &Task| {
                    log.lock().unwrap().push(t.name.clone());
                    orig(t)
                })
            })
            .zipmap(|total: &u64| if *total >= 500 { "critical" } else { "ok" });

        let (total, category) = Sequential.run(&pipeline, &graph, &root);
        let names: Vec<String> = visited.lock().unwrap().clone();
    // ANCHOR_END: composed
        assert_eq!(total, 800);
        assert_eq!(category, "critical");
        assert_snapshot!("composed", format!(
            "total={total} [{category}], visited: {}", names.join(" → ")
        ));
    }
}

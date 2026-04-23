//! Parallel execution: Fused vs Funnel over flat data.
//! Demonstrates: adjacency-list graph, identical results across
//! policies, session scope, explicit pool attach.

#[cfg(test)]
mod tests {
    use hylic::domain::shared as dom;
    use hylic::graph;
    use hylic::exec::funnel;
    use insta::assert_snapshot;

    /// Build a tree as a flat adjacency list + value array.
    /// Node 0 is the root with 6 children; each child has 3 leaves.
    fn build_tree() -> (Vec<Vec<usize>>, Vec<u64>) {
        let mut adj: Vec<Vec<usize>> = Vec::new();
        let mut vals: Vec<u64> = Vec::new();

        // root (node 0)
        adj.push((1..=6).collect());
        vals.push(1);

        // 6 branches (nodes 1-6), each with 3 leaves
        let mut next_leaf = 7;
        for i in 0..6 {
            let children: Vec<usize> = (next_leaf..next_leaf + 3).collect();
            adj.push(children);
            vals.push(i as u64 * 10);
            next_leaf += 3;
        }

        // 18 leaves (nodes 7-24)
        for i in 0..6 {
            for j in 0..3u64 {
                adj.push(vec![]);
                vals.push(i as u64 * 10 + j);
            }
        }

        (adj, vals)
    }

    #[test]
    fn parallel_strategies() {
        let (adj, vals) = build_tree();

        // The treeish looks up children by index — no nested structs
        let adj_for_graph = adj.clone();
        let graph = graph::treeish_visit(move |n: &usize, cb: &mut dyn FnMut(&usize)| {
            for &c in &adj_for_graph[*n] { cb(&c); }
        });

        let vals_for_fold = vals.clone();
        let sum = dom::fold(
            move |n: &usize| vals_for_fold[*n],
            |heap: &mut u64, child: &u64| *heap += child,
            |heap: &u64| *heap,
        );

        // Sequential baseline
        let expected = dom::FUSED.run(&sum, &graph, &0usize);

        // One-shot: .run() creates + destroys pool internally
        let r_default = dom::exec(funnel::Spec::default(4)).run(&sum, &graph, &0usize);
        assert_eq!(r_default, expected);

        // Different policy: wide-light
        let r_wide = dom::exec(funnel::Spec::for_wide_light(4)).run(&sum, &graph, &0usize);
        assert_eq!(r_wide, expected);

        // Session scope: pool shared across folds
        dom::exec(funnel::Spec::default(4)).session(|s| {
            assert_eq!(s.run(&sum, &graph, &0usize), expected);
            assert_eq!(s.run(&sum, &graph, &0usize), expected);
        });

        // Explicit attach: manual pool, multiple policies
        funnel::Pool::with(4, |pool| {
            let pw = dom::exec(funnel::Spec::default(4)).attach(pool);
            let sh = dom::exec(funnel::Spec::for_wide_light(4)).attach(pool);
            assert_eq!(pw.run(&sum, &graph, &0usize), expected);
            assert_eq!(sh.run(&sum, &graph, &0usize), expected);
        });

        assert_snapshot!("parallel", format!(
            "sum = {expected}, verified: fused, funnel(one-shot), funnel(wide), session, attach"
        ));
    }
}

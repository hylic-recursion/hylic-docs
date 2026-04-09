//! Parallel execution: Fused vs Funnel.
//! Demonstrates: identical results, policy variants, session scope, attach.

#[cfg(test)]
mod tests {
    use hylic::domain::shared as dom;
    use hylic::cata::exec::funnel;
    use insta::assert_snapshot;

    #[derive(Clone)]
    struct WorkNode {
        value: u64,
        children: Vec<WorkNode>,
    }

    impl WorkNode {
        fn leaf(v: u64) -> Self { WorkNode { value: v, children: vec![] } }
        fn branch(v: u64, ch: Vec<WorkNode>) -> Self { WorkNode { value: v, children: ch } }
    }

    #[test]
    fn parallel_strategies() {
        let tree = WorkNode::branch(1, (0..6).map(|i|
            WorkNode::branch(i * 10, (0..3).map(|j|
                WorkNode::leaf(i * 10 + j)
            ).collect())
        ).collect());

        let graph = dom::treeish(|n: &WorkNode| n.children.clone());
        let init = |n: &WorkNode| n.value;
        let acc = |heap: &mut u64, child: &u64| *heap += child;
        let sum = dom::simple_fold(init, acc);

        // Sequential baseline
        let expected = dom::FUSED.run(&sum, &graph, &tree);

        // One-shot: .run() creates + destroys pool internally
        let r_default = dom::exec(funnel::Spec::default(4)).run(&sum, &graph, &tree);
        assert_eq!(r_default, expected);

        // Different policy: wide-light
        let r_wide = dom::exec(funnel::Spec::for_wide_light(4)).run(&sum, &graph, &tree);
        assert_eq!(r_wide, expected);

        // Session scope: pool shared across folds
        dom::exec(funnel::Spec::default(4)).session(|s| {
            assert_eq!(s.run(&sum, &graph, &tree), expected);
            assert_eq!(s.run(&sum, &graph, &tree), expected);
        });

        // Explicit attach: manual pool, multiple policies
        funnel::Pool::with(4, |pool| {
            let pw = dom::exec(funnel::Spec::default(4)).attach(pool);
            let sh = dom::exec(funnel::Spec::for_wide_light(4)).attach(pool);
            assert_eq!(pw.run(&sum, &graph, &tree), expected);
            assert_eq!(sh.run(&sum, &graph, &tree), expected);
        });

        assert_snapshot!("parallel", format!(
            "sum = {expected}, verified: fused, funnel(one-shot), funnel(wide), session, attach"
        ));
    }
}

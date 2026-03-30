//! Parallel execution: same fold, different strategies.
//! Demonstrates: Strategy enum, verifying equivalence across modes.

#[cfg(test)]
mod tests {
    use hylic::fold::simple_fold;
    use hylic::graph::treeish;
    use hylic::cata::{Strategy, ALL};
    use insta::assert_snapshot;


    /// A computation node with configurable work per node.
    /// The tree is wide (high branching factor) to benefit from parallelism.
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
        // Wide tree: root with 6 children, each with 3 leaves.
        let tree = WorkNode::branch(1, (0..6).map(|i|
            WorkNode::branch(i * 10, (0..3).map(|j|
                WorkNode::leaf(i * 10 + j)
            ).collect())
        ).collect());

        let graph = treeish(|n: &WorkNode| n.children.clone());

        let sum = simple_fold(
            |n: &WorkNode| n.value,
            |heap: &mut u64, child: &u64| *heap += child,
        );

        // All strategies produce the same result.
        let expected = Strategy::Sequential.run(&sum, &graph, &tree);
        for strategy in ALL {
            let result = strategy.run(&sum, &graph, &tree);
            assert_eq!(result, expected, "Strategy {:?} disagreed", strategy);
        }

        assert_snapshot!("parallel", format!(
            "sum = {expected}, verified across {} strategies", ALL.len()
        ));
    }
}

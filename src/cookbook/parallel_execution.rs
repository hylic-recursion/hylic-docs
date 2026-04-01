//! Parallel execution: same fold, different executors.
//! Demonstrates: Exec constructors produce identical results.

#[cfg(test)]
mod tests {
    use hylic::fold::simple_fold;
    use hylic::graph::treeish;
    use hylic::cata::exec::{self, Executor, ExecutorExt};
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

        let graph = treeish(|n: &WorkNode| n.children.clone());
        let init = |n: &WorkNode| n.value;
        let acc = |heap: &mut u64, child: &u64| *heap += child;
        let sum = simple_fold(init, acc);

        // All executors produce identical results.
        let executors: Vec<exec::Exec<WorkNode, u64>> = vec![
            exec::Exec::fused(),
            exec::Exec::sequential(),
            exec::Exec::rayon(),
        ];
        let expected = executors[0].run(&sum, &graph, &tree);
        for exec in &executors {
            assert_eq!(exec.run(&sum, &graph, &tree), expected);
        }

        assert_snapshot!("parallel", format!(
            "sum = {expected}, verified across {} executors", executors.len()
        ));
    }
}

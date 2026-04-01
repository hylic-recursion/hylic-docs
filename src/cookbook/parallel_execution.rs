//! Parallel execution: same fold, different executors.
//! Demonstrates: Exec constructors produce identical results.

#[cfg(test)]
mod tests {
    use hylic::fold::simple_fold;
    use hylic::graph::treeish;
    use hylic::cata::{Exec, Fused, Rayon, Executor};
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
        let sum = simple_fold(
            |n: &WorkNode| n.value,
            |heap: &mut u64, child: &u64| *heap += child,
        );

        // All executors produce identical results.
        let executors: Vec<Exec<WorkNode, u64>> = vec![
            Fused.into(),
            Exec::sequential(),
            Rayon.into(),
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

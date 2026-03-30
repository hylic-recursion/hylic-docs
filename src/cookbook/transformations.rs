//! Fold transformations: logging, zipmap, memoization.
//! Demonstrates: map_init for side effects, zipmap for derived annotations,
//! memoize_treeish for caching graph traversal.

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use hylic::fold::simple_fold;
    use hylic::graph::treeish_from;
    use hylic::cata::Strategy;
    use insta::assert_snapshot;

    // ANCHOR: transformations

    #[derive(Clone, Debug)]
    struct Node {
        name: String,
        value: i32,
        children: Vec<Node>,
    }

    impl Node {
        fn leaf(name: &str, value: i32) -> Self {
            Node { name: name.into(), value, children: vec![] }
        }
        fn branch(name: &str, value: i32, ch: Vec<Node>) -> Self {
            Node { name: name.into(), value, children: ch }
        }
    }

    #[test]
    fn logging_via_map_init() {
        let tree = Node::branch("root", 1, vec![
            Node::branch("a", 2, vec![Node::leaf("a1", 3)]),
            Node::leaf("b", 4),
        ]);

        let graph = treeish_from(|n: &Node| n.children.as_slice());

        let sum = simple_fold(
            |n: &Node| n.value,
            |heap: &mut i32, child: &i32| *heap += child,
        );

        // map_init wraps the init phase to add logging.
        let log = Arc::new(Mutex::new(Vec::new()));
        let log_clone = log.clone();
        let logged_sum = sum.map_init(move |original_init| {
            Box::new(move |node: &Node| {
                log_clone.lock().unwrap().push(node.name.clone());
                original_init(node)
            })
        });

        let result = Strategy::Sequential.run(&logged_sum, &graph, &tree);
        let visited: Vec<String> = log.lock().unwrap().clone();

        assert_eq!(result, 10);

        // ANCHOR_END: transformations
        assert_snapshot!("logged", format!(
            "sum = {result}, visited: {}", visited.join(" → ")
        ));
    }

    // ANCHOR: zipmap
    /// zipmap derives per-node annotations from the fold result.
    /// Here: classify each subtree by whether its sum exceeds a threshold,
    /// producing a tree of (sum, category) pairs.
    #[test]
    fn zipmap_classify_subtrees() {
        let tree = Node::branch("root", 1, vec![
            Node::branch("heavy", 10, vec![
                Node::leaf("h1", 20),
                Node::leaf("h2", 15),
            ]),
            Node::leaf("light", 2),
        ]);

        let graph = treeish_from(|n: &Node| n.children.as_slice());

        let sum = simple_fold(
            |n: &Node| n.value,
            |heap: &mut i32, child: &i32| *heap += child,
        );

        // zipmap: each node's sum gets paired with a classification.
        // The classification is per-node — derived from that subtree's total.
        let classified = sum.zipmap(|total: &i32| {
            if *total >= 20 { "critical" }
            else if *total >= 5 { "moderate" }
            else { "negligible" }
        });

        let (total, category) = Strategy::Sequential.run(&classified, &graph, &tree);
        assert_eq!(total, 48);
        assert_eq!(category, "critical");
        // ANCHOR_END: zipmap

        assert_snapshot!("zipmap", format!("total = {total}, category = {category}"));
    }
}

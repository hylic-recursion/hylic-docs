//! Cycle detection in a dependency graph.
//! Demonstrates: treeish over a graph with potential cycles,
//! fold that tracks visited nodes to detect re-entry.

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use hylic::fold::simple_fold;
    use hylic::graph::treeish;
    use hylic::cata::{Fused, Executor};
    use insta::assert_snapshot;


    /// A dependency graph defined as adjacency lists.
    /// Nodes are string IDs, edges are dependencies.
    #[derive(Clone)]
    struct DepGraph {
        edges: HashMap<String, Vec<String>>,
    }

    impl DepGraph {
        fn new(edges: &[(&str, &[&str])]) -> Self {
            DepGraph {
                edges: edges.iter()
                    .map(|(k, v)| (k.to_string(), v.iter().map(|s| s.to_string()).collect()))
                    .collect(),
            }
        }
    }

    /// A node in the traversal: carries the current ID and
    /// the set of ancestors on this path (for cycle detection).
    #[derive(Clone)]
    struct DepNode {
        id: String,
        ancestors: HashSet<String>,
    }

    impl DepNode {
        fn root(id: &str) -> Self {
            DepNode { id: id.to_string(), ancestors: HashSet::new() }
        }
        fn child(&self, id: &str) -> Self {
            let mut ancestors = self.ancestors.clone();
            ancestors.insert(self.id.clone());
            DepNode { id: id.to_string(), ancestors }
        }
        fn is_cycle(&self) -> bool {
            self.ancestors.contains(&self.id)
        }
    }

    /// Result of cycle analysis for a subtree.
    #[derive(Clone, Debug)]
    struct CycleResult {
        cycles: Vec<String>,
        visited: usize,
    }

    #[test]
    fn detect_cycles() {
        let graph_data = DepGraph::new(&[
            ("A", &["B", "C"]),
            ("B", &["D"]),
            ("C", &["D", "A"]),  // C → A creates a cycle
            ("D", &[]),
        ]);

        // The cycle state lives in the NODE TYPE (DepNode carries its ancestor set),
        // not in the fold. The treeish decides what to traverse: cycles become
        // leaves (empty children), stopping recursion. The fold just collects.
        // This is the hylic pattern: structure decisions in Treeish, computation in Fold.
        let graph = treeish(move |node: &DepNode| {
            if node.is_cycle() { return vec![]; }
            graph_data.edges.get(&node.id)
                .map(|deps| deps.iter().map(|d| node.child(d)).collect())
                .unwrap_or_default()
        });

        // Fold: collect cycles from leaves, count visited nodes.
        let detect = simple_fold(
            |node: &DepNode| CycleResult {
                cycles: if node.is_cycle() { vec![node.id.clone()] } else { vec![] },
                visited: 1,
            },
            |heap: &mut CycleResult, child: &CycleResult| {
                heap.cycles.extend(child.cycles.iter().cloned());
                heap.visited += child.visited;
            },
        );

        let result = Fused.run(&detect, &graph, &DepNode::root("A"));

        assert_eq!(result.cycles, vec!["A"]);  // C → A cycle detected
        assert_eq!(result.visited, 6);          // A, B, C, D, D, A(cycle)

        assert_snapshot!("cycles", format!(
            "cycles: {:?}, visited: {}", result.cycles, result.visited
        ));
    }
}

//! Cycle detection in a dependency graph.
//! Demonstrates: treeish over a graph with potential cycles,
//! fold that tracks visited nodes to detect re-entry.

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use hylic::prelude::*;
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

        // Cycle state lives in the node type — DepNode carries its ancestor
        // set. When a node sees itself in that set, the treeish stops by
        // returning no children.
        let graph: Treeish<DepNode> = treeish(move |node: &DepNode| {
            if node.is_cycle() { return vec![]; }
            graph_data.edges.get(&node.id)
                .map(|deps| deps.iter().map(|d| node.child(d)).collect())
                .unwrap_or_default()
        });

        let detect: Fold<DepNode, CycleResult, CycleResult> = fold(
            |node: &DepNode| CycleResult {
                cycles:  if node.is_cycle() { vec![node.id.clone()] } else { vec![] },
                visited: 1,
            },
            |heap: &mut CycleResult, child: &CycleResult| {
                heap.cycles.extend(child.cycles.iter().cloned());
                heap.visited += child.visited;
            },
            |h: &CycleResult| h.clone(),
        );

        let result: CycleResult = FUSED.run(&detect, &graph, &DepNode::root("A"));

        assert_eq!(result.cycles, vec!["A"]);  // C → A cycle detected
        assert_eq!(result.visited, 6);          // A, B, C, D, D, A(cycle)

        assert_snapshot!("cycles", format!(
            "cycles: {:?}, visited: {}", result.cycles, result.visited
        ));
    }
}

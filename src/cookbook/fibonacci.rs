//! Fibonacci via tree fold — the simplest hylic example.

#[cfg(test)]
mod tests {
    use hylic::fold::simple_fold;
    use hylic::graph::treeish;
    use hylic::cata::exec::{self, Executor};
    use insta::assert_snapshot;


    /// A Fibonacci node: just the number n.
    /// Branches into n-1 and n-2 until reaching base cases 0 or 1.
    #[derive(Clone)]
    struct FibNode(u64);

    #[test]
    fn fibonacci() {
        // treeish: given a node, return its children.
        // Leaves (n <= 1) have no children — empty vec stops recursion.
        let graph = treeish(|n: &FibNode| {
            if n.0 <= 1 { vec![] }
            else { vec![FibNode(n.0 - 1), FibNode(n.0 - 2)] }
        });

        // simple_fold: H = R (heap IS the result, finalize is clone).
        // init: each node seeds its heap — leaves get their value, inner nodes get 0.
        // accumulate: called once per child result, folds it into the heap.
        let init = |n: &FibNode| if n.0 <= 1 { n.0 } else { 0 };
        let acc = |heap: &mut u64, child: &u64| *heap += child;
        let fib = simple_fold(init, acc);

        let result = exec::FUSED.run(&fib, &graph, &FibNode(10));
        assert_eq!(result, 55);

        assert_snapshot!("fib10", format!("fib(10) = {result}"));
    }
}

//! Fibonacci via tree fold — the simplest hylic example.

#[cfg(test)]
mod tests {
    use hylic::fold::simple_fold;
    use hylic::graph::treeish;
    use hylic::cata::Strategy;
    use insta::assert_snapshot;


    /// A Fibonacci node: just the number n.
    /// Branches into n-1 and n-2 until reaching base cases 0 or 1.
    #[derive(Clone)]
    struct FibNode(u64);

    #[test]
    fn fibonacci() {
        // Tree structure: each node > 1 has two children.
        let graph = treeish(|n: &FibNode| {
            if n.0 <= 1 { vec![] }
            else { vec![FibNode(n.0 - 1), FibNode(n.0 - 2)] }
        });

        // Fold: leaves contribute their value, inner nodes sum children.
        let fib = simple_fold(
            |n: &FibNode| if n.0 <= 1 { n.0 } else { 0 },
            |heap: &mut u64, child: &u64| *heap += child,
        );

        let result = Strategy::Sequential.run(&fib, &graph, &FibNode(10));
        assert_eq!(result, 55);

        assert_snapshot!("fib10", format!("fib(10) = {result}"));
    }
}

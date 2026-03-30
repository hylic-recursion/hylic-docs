//! Fibonacci via tree fold — the simplest hylic example.

#[cfg(test)]
mod tests {
    use hylic::fold::simple_fold;
    use hylic::graph::treeish;
    use hylic::cata::Strategy;
    use insta::assert_snapshot;

    #[derive(Clone)]
    struct FibNode(u64);

    #[test]
    fn fibonacci() {
        // ANCHOR: fibonacci
        let graph = treeish(|n: &FibNode| {
            if n.0 <= 1 { vec![] }
            else { vec![FibNode(n.0 - 1), FibNode(n.0 - 2)] }
        });

        let fib_fold = simple_fold(
            |n: &FibNode| if n.0 <= 1 { n.0 } else { 0 },
            |heap: &mut u64, child: &u64| *heap += child,
        );

        let result = Strategy::Sequential.run(&fib_fold, &graph, &FibNode(10));
        // ANCHOR_END: fibonacci

        assert_eq!(result, 55);
        assert_snapshot!("fibonacci_result", format!("fib(10) = {result}"));
    }
}

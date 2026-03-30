//! # Fibonacci via tree fold
//!
//! The simplest possible hylic example. Fibonacci numbers are a degenerate
//! tree (each node has exactly two children: n-1 and n-2). The fold sums them.
//!
//! This is intentionally naive (exponential) — it demonstrates the mechanics,
//! not performance. For efficient Fibonacci, don't use tree recursion.

#[cfg(test)]
mod tests {
    use hylic::fold::simple_fold;
    use hylic::graph::treeish;
    use hylic::cata::Strategy;

    #[derive(Clone)]
    struct FibNode(u64);

    #[test]
    fn fibonacci() {
        let graph = treeish(|n: &FibNode| {
            if n.0 <= 1 { vec![] }
            else { vec![FibNode(n.0 - 1), FibNode(n.0 - 2)] }
        });

        let fib_fold = simple_fold(
            |n: &FibNode| if n.0 <= 1 { n.0 } else { 0 },
            |heap: &mut u64, child: &u64| *heap += child,
        );

        let result = Strategy::Sequential.run(&fib_fold, &graph, &FibNode(10));
        assert_eq!(result, 55);
        eprintln!("fib(10) = {result}");
    }
}

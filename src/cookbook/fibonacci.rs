//! Fibonacci via tree fold — the simplest hylic example.
//! The node type is `i32` — not a struct with children.
//! The treeish computes children from the value: fib(n) → [fib(n-1), fib(n-2)].

#[cfg(test)]
mod tests {
    use hylic::prelude::*;
    use insta::assert_snapshot;


    /// A Fibonacci node: just the number n.
    /// Branches into n-1 and n-2 until reaching base cases 0 or 1.
    #[derive(Clone)]
    struct FibNode(u64);

    #[test]
    fn fibonacci() {
        // Children of fib(n) are fib(n-1) and fib(n-2); fib(0) and fib(1) are leaves.
        let graph: Treeish<FibNode> = treeish(|n: &FibNode| {
            if n.0 <= 1 { vec![] }
            else { vec![FibNode(n.0 - 1), FibNode(n.0 - 2)] }
        });

        // init: leaves seed the heap with n; inner nodes seed with 0.
        // accumulate: each child's result is summed into the heap.
        // finalize: identity (H = R = u64).
        let fib: Fold<FibNode, u64, u64> = fold(
            |n: &FibNode| if n.0 <= 1 { n.0 } else { 0 },
            |heap: &mut u64, child: &u64| *heap += child,
            |h: &u64| *h,
        );

        let result: u64 = FUSED.run(&fib, &graph, &FibNode(10));
        assert_eq!(result, 55);

        assert_snapshot!("fib10", format!("fib(10) = {result}"));
    }
}

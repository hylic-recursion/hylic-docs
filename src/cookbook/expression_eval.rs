//! Expression evaluation — AST fold with heterogeneous node types.

#[cfg(test)]
mod tests {
    use hylic::prelude::vec_fold::{vec_fold, VecHeap};
    use hylic::domain::shared as dom;
    use insta::assert_snapshot;


    /// An arithmetic expression tree.
    /// Each variant defines both its meaning and its children.
    #[derive(Clone)]
    enum Expr {
        Num(f64),
        Add(Box<Expr>, Box<Expr>),
        Mul(Box<Expr>, Box<Expr>),
        Neg(Box<Expr>),
    }

    /// Convenience constructors for readable test data.
    fn num(v: f64) -> Expr { Expr::Num(v) }
    fn add(a: Expr, b: Expr) -> Expr { Expr::Add(Box::new(a), Box::new(b)) }
    fn mul(a: Expr, b: Expr) -> Expr { Expr::Mul(Box::new(a), Box::new(b)) }
    fn neg(a: Expr) -> Expr { Expr::Neg(Box::new(a)) }

    #[test]
    fn evaluate_expression() {
        let expr = mul(add(num(3.0), num(4.0)), neg(num(2.0)));

        // treeish_visit: callback-based traversal — no Vec allocation.
        // Each variant decides which children to visit.
        let graph = dom::treeish_visit(|e: &Expr, cb: &mut dyn FnMut(&Expr)| {
            match e {
                Expr::Num(_) => {}
                Expr::Add(a, b) | Expr::Mul(a, b) => { cb(a); cb(b); }
                Expr::Neg(a) => { cb(a); }
            }
        });

        // vec_fold: unlike simple_fold, finalize sees the node AND all child
        // results together. Needed here because each node type combines
        // children differently (sum vs product vs negate).
        let format = |heap: &VecHeap<Expr, f64>| {
            match &heap.node {
                Expr::Num(v) => *v,
                Expr::Add(_, _) => heap.childresults.iter().sum(),
                Expr::Mul(_, _) => heap.childresults.iter().product(),
                Expr::Neg(_) => -heap.childresults[0],
            }
        };
        let eval = vec_fold(format);

        let result = dom::FUSED.run(&eval, &graph, &expr);
        assert_eq!(result, -14.0);

        assert_snapshot!("expr_eval", format!("(3 + 4) * -(2) = {result}"));
    }
}

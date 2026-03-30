//! Expression evaluation — AST fold with heterogeneous node types.

#[cfg(test)]
mod tests {
    use hylic::prelude::vec_fold::{vec_fold, VecHeap};
    use hylic::graph::treeish_visit;
    use hylic::cata::Strategy;
    use insta::assert_snapshot;

    // ANCHOR: expression_eval

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

        // Tree structure: visit each child by reference.
        let graph = treeish_visit(|e: &Expr, cb: &mut dyn FnMut(&Expr)| {
            match e {
                Expr::Num(_) => {}
                Expr::Add(a, b) | Expr::Mul(a, b) => { cb(a); cb(b); }
                Expr::Neg(a) => { cb(a); }
            }
        });

        // Fold: vec_fold sees the node + all child results.
        // Each node type combines results differently.
        let eval = vec_fold(|heap: &VecHeap<Expr, f64>| {
            match &heap.node {
                Expr::Num(v) => *v,
                Expr::Add(_, _) => heap.childresults.iter().sum(),
                Expr::Mul(_, _) => heap.childresults.iter().product(),
                Expr::Neg(_) => -heap.childresults[0],
            }
        });

        let result = Strategy::Sequential.run(&eval, &graph, &expr);
        assert_eq!(result, -14.0);

        // ANCHOR_END: expression_eval
        assert_snapshot!("expr_eval", format!("(3 + 4) * -(2) = {result}"));
    }
}

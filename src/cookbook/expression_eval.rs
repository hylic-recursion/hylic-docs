//! # Expression evaluation (AST fold)
//!
//! A classic: fold an arithmetic expression tree bottom-up.
//! Demonstrates how any AST with heterogeneous node types maps onto
//! hylic's Treeish + Fold pattern.

#[cfg(test)]
mod tests {
    use hylic::fold::simple_fold;
    use hylic::graph::treeish_visit;
    use hylic::cata::Strategy;

    #[derive(Clone)]
    enum Expr {
        Num(f64),
        Add(Box<Expr>, Box<Expr>),
        Mul(Box<Expr>, Box<Expr>),
        Neg(Box<Expr>),
    }

    fn num(v: f64) -> Expr { Expr::Num(v) }
    fn add(a: Expr, b: Expr) -> Expr { Expr::Add(Box::new(a), Box::new(b)) }
    fn mul(a: Expr, b: Expr) -> Expr { Expr::Mul(Box::new(a), Box::new(b)) }
    fn neg(a: Expr) -> Expr { Expr::Neg(Box::new(a)) }

    #[test]
    fn evaluate_expression() {
        // (3 + 4) * -(2)
        let expr = mul(add(num(3.0), num(4.0)), neg(num(2.0)));

        let graph = treeish_visit(|e: &Expr, cb: &mut dyn FnMut(&Expr)| {
            match e {
                Expr::Num(_) => {}
                Expr::Add(a, b) | Expr::Mul(a, b) => { cb(a); cb(b); }
                Expr::Neg(a) => { cb(a); }
            }
        });

        // init: leaf value or identity for the operation
        // accumulate: combine child results based on node type
        let eval = simple_fold(
            |e: &Expr| match e {
                Expr::Num(v) => *v,
                Expr::Add(_, _) => 0.0,
                Expr::Mul(_, _) => 1.0,
                Expr::Neg(_) => 0.0,
            },
            |heap: &mut f64, child: &f64| {
                // This works because add's identity is 0 (0+a+b = a+b)
                // and mul's identity is 1 (1*a*b = a*b).
                // Neg is special: it negates whatever its single child produced.
                *heap += child; // for add and neg
            },
        );

        // The simple_fold above doesn't distinguish add from mul in accumulate.
        // For proper evaluation, use vec_fold which sees the node + all children:
        let eval_proper = hylic::prelude::vec_fold::vec_fold(
            |heap: &hylic::prelude::vec_fold::VecHeap<Expr, f64>| {
                match &heap.node {
                    Expr::Num(v) => *v,
                    Expr::Add(_, _) => heap.childresults.iter().sum(),
                    Expr::Mul(_, _) => heap.childresults.iter().product(),
                    Expr::Neg(_) => -heap.childresults.first().unwrap_or(&0.0),
                }
            },
        );

        let result = Strategy::Sequential.run(&eval_proper, &graph, &expr);
        assert_eq!(result, -14.0); // (3+4) * -(2) = 7 * -2 = -14
        eprintln!("(3 + 4) * -(2) = {result}");
    }
}

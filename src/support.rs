//! Shared display helpers for cookbook examples.
use hylic::domain::shared as dom;
use hylic::graph;

/// Run a fold and print the result with a label.
pub fn show<N: 'static, H: 'static, R: std::fmt::Debug + 'static>(
    label: &str,
    fold: &dom::Fold<N, H, R>,
    graph: &graph::Treeish<N>,
    root: &N,
) {
    let result = dom::FUSED.run(fold, graph, root);
    eprintln!("{label}: {result:?}");
}

/// Run a fold with fused and funnel executors, assert they agree.
pub fn show_all_exec<N, H, R: std::fmt::Debug + PartialEq>(
    label: &str,
    fold: &dom::Fold<N, H, R>,
    graph: &graph::Treeish<N>,
    root: &N,
) where
    N: Clone + Send + 'static,
    H: 'static,
    R: Clone + Send + 'static + PartialEq,
{
    use hylic::exec::funnel;
    let expected = dom::FUSED.run(fold, graph, root);
    let funnel_spec = funnel::Spec::default(4);
    let funnel_result = dom::exec(funnel_spec).run(fold, graph, root);
    assert_eq!(funnel_result, expected, "Funnel disagreed");
    eprintln!("{label}: {expected:?}");
}

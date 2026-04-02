//! Shared display helpers for cookbook examples.
use hylic::domain::shared as dom;

/// Run a fold and print the result with a label.
pub fn show<N: 'static, H: 'static, R: std::fmt::Debug + 'static>(
    label: &str,
    fold: &dom::Fold<N, H, R>,
    graph: &dom::Treeish<N>,
    root: &N,
) {
    let result = dom::FUSED.run(fold, graph, root);
    eprintln!("{label}: {result:?}");
}

/// Run a fold with fused and rayon executors, assert they agree.
pub fn show_all_exec<N, H, R: std::fmt::Debug + PartialEq>(
    label: &str,
    fold: &dom::Fold<N, H, R>,
    graph: &dom::Treeish<N>,
    root: &N,
) where
    N: Clone + Send + Sync + 'static,
    H: Send + Sync + 'static,
    R: Clone + Send + Sync + 'static + PartialEq,
{
    let expected = dom::FUSED.run(fold, graph, root);
    let rayon_result = dom::RAYON.run(fold, graph, root);
    assert_eq!(rayon_result, expected, "Rayon disagreed");
    eprintln!("{label}: {expected:?}");
}

//! Shared display helpers for cookbook examples.

use hylic::fold::Fold;
use hylic::graph::Treeish;
use hylic::cata::{Fused, Rayon, Executor};

/// Run a fold and print the result with a label.
/// Uses Fused directly — no Send+Sync bounds needed.
pub fn show<N: 'static, H: 'static, R: std::fmt::Debug + 'static>(
    label: &str,
    fold: &Fold<N, H, R>,
    graph: &Treeish<N>,
    root: &N,
) {
    let result = Fused.run(fold, graph, root);
    eprintln!("{label}: {result:?}");
}

/// Run a fold with fused and rayon executors, assert they agree.
pub fn show_all_exec<N, H, R: std::fmt::Debug + PartialEq>(
    label: &str,
    fold: &Fold<N, H, R>,
    graph: &Treeish<N>,
    root: &N,
) where
    N: Clone + Send + Sync + 'static,
    H: Send + Sync + 'static,
    R: Clone + Send + Sync + 'static + PartialEq,
{
    let expected = Fused.run(fold, graph, root);
    let rayon_result = Rayon.run(fold, graph, root);
    assert_eq!(rayon_result, expected, "Rayon disagreed");
    eprintln!("{label}: {expected:?}");
}

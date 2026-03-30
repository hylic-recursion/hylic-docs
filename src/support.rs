//! Shared display helpers for cookbook examples.
//! Demonstrates how to build domain-specific formatters on hylic.

use hylic::fold::Fold;
use hylic::graph::Treeish;
use hylic::cata::Strategy;

/// Run a fold and print the result with a label.
pub fn show<N, H, R: std::fmt::Debug>(
    label: &str,
    fold: &Fold<N, H, R>,
    graph: &Treeish<N>,
    root: &N,
) where
    N: Clone + Send + Sync + 'static,
    H: Send + Sync + 'static,
    R: Clone + Send + Sync + 'static,
{
    let result = Strategy::Sequential.run(fold, graph, root);
    eprintln!("{label}: {result:?}");
}

/// Run a fold with all strategies, assert they agree, print the result.
pub fn show_all_strategies<N, H, R: std::fmt::Debug + PartialEq>(
    label: &str,
    fold: &Fold<N, H, R>,
    graph: &Treeish<N>,
    root: &N,
) where
    N: Clone + Send + Sync + 'static,
    H: Send + Sync + 'static,
    R: Clone + Send + Sync + 'static + PartialEq,
{
    let expected = Strategy::Sequential.run(fold, graph, root);
    for s in hylic::cata::ALL {
        let result = s.run(fold, graph, root);
        assert_eq!(result, expected, "Strategy {:?} disagreed", s);
    }
    eprintln!("{label}: {expected:?}");
}

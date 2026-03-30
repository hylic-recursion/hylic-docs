//! Configuration inheritance with overlay/merge.
//! Demonstrates: a fold where the heap IS a config map,
//! and children's configs overlay the parent's defaults.

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use hylic::fold::simple_fold;
    use hylic::graph::treeish_from;
    use hylic::cata::Strategy;
    use insta::assert_snapshot;


    /// A configuration scope. Each scope has its own key-value overrides
    /// and child scopes that inherit and can further override.
    #[derive(Clone, Debug)]
    struct ConfigScope {
        name: String,
        overrides: BTreeMap<String, String>,
        children: Vec<ConfigScope>,
    }

    impl ConfigScope {
        fn new(name: &str, overrides: &[(&str, &str)], children: Vec<ConfigScope>) -> Self {
            ConfigScope {
                name: name.into(),
                overrides: overrides.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
                children,
            }
        }
        fn leaf(name: &str, overrides: &[(&str, &str)]) -> Self {
            Self::new(name, overrides, vec![])
        }
    }

    /// Resolved configuration: the merged key-value map for a scope,
    /// collecting all overrides from the scope and its descendants.
    #[derive(Clone, Debug, PartialEq)]
    struct ResolvedConfig {
        scope: String,
        merged: BTreeMap<String, String>,
    }

    #[test]
    fn config_overlay() {
        let root = ConfigScope::new("global", &[
            ("color", "blue"),
            ("font_size", "12"),
            ("theme", "light"),
        ], vec![
            ConfigScope::new("production", &[
                ("theme", "dark"),
                ("debug", "false"),
            ], vec![
                ConfigScope::leaf("production.api", &[
                    ("font_size", "14"),
                    ("rate_limit", "1000"),
                ]),
            ]),
            ConfigScope::leaf("development", &[
                ("debug", "true"),
                ("theme", "light"),
            ]),
        ]);

        let graph = treeish_from(|scope: &ConfigScope| scope.children.as_slice());

        // Fold: each node starts with its own overrides.
        // Children's merged configs bubble up — but we want
        // parent-wins semantics (parent overrides beat children).
        // So we merge children first, then overlay parent's own.
        let resolve = simple_fold(
            |scope: &ConfigScope| ResolvedConfig {
                scope: scope.name.clone(),
                merged: scope.overrides.clone(),
            },
            |heap: &mut ResolvedConfig, child: &ResolvedConfig| {
                // Child values only fill in keys the parent hasn't set.
                for (k, v) in &child.merged {
                    heap.merged.entry(k.clone()).or_insert_with(|| v.clone());
                }
            },
        );

        let result = Strategy::Sequential.run(&resolve, &graph, &root);

        // Global scope sees all keys from all descendants,
        // but its own values win for "color", "font_size", "theme".
        assert_eq!(result.merged.get("color").unwrap(), "blue");
        assert_eq!(result.merged.get("theme").unwrap(), "light");  // parent wins
        assert_eq!(result.merged.get("debug").unwrap(), "false");  // production's value
        assert_eq!(result.merged.get("rate_limit").unwrap(), "1000");

        let display: Vec<String> = result.merged.iter()
            .map(|(k, v)| format!("{k}={v}")).collect();
        assert_snapshot!("config", display.join(", "));
    }
}

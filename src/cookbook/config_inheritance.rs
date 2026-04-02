//! Configuration inheritance with overlay/merge.
//! Demonstrates: a fold where the heap IS a config map,
//! and children's configs overlay the parent's defaults.

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use hylic::domain::shared as dom;
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

        // treeish_from: for structs with a children field — zero-clone slice access.
        let graph = dom::treeish_from(|scope: &ConfigScope| scope.children.as_slice());

        // init seeds the heap with the parent's own overrides.
        // accumulate merges each child's resolved config upward.
        // or_insert means: child values only fill in keys the parent hasn't set.
        // This gives parent-wins semantics — init runs before accumulate.
        let init = |scope: &ConfigScope| ResolvedConfig {
            scope: scope.name.clone(),
            merged: scope.overrides.clone(),
        };
        let acc = |heap: &mut ResolvedConfig, child: &ResolvedConfig| {
            // Child values only fill in keys the parent hasn't set.
            for (k, v) in &child.merged {
                heap.merged.entry(k.clone()).or_insert_with(|| v.clone());
            }
        };
        let resolve = dom::simple_fold(init, acc);

        let result = dom::FUSED.run(&resolve, &graph, &root);

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

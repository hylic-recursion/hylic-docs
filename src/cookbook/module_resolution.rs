//! Minified module resolution — the pattern that motivated hylic.
//! Demonstrates: SeedGraph (entry point differs from recursion),
//! error handling via Either, and the "two-function pattern" solved.

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use either::Either;

    use hylic::domain::shared::GraphWithFold;
    use hylic::prelude::seeds_for_fallible;
    use hylic::domain::shared as dom;
    use insta::assert_snapshot;


    /// A module has a name and declares dependencies on other modules.
    #[derive(Clone, Debug)]
    struct Module {
        name: String,
        deps: Vec<String>,
    }

    /// A module registry: maps names to module definitions.
    /// In reality this would be filesystem lookups.
    struct Registry(HashMap<String, Module>);

    impl Registry {
        fn new(modules: &[(&str, &[&str])]) -> Self {
            Registry(modules.iter().map(|(name, deps)| {
                (name.to_string(), Module {
                    name: name.to_string(),
                    deps: deps.iter().map(|s| s.to_string()).collect(),
                })
            }).collect())
        }
    }

    /// Error when a module can't be found.
    #[derive(Clone, Debug)]
    struct ResolveError(String);

    /// Resolution result: either an error or a list of resolved module names.
    #[derive(Clone, Debug)]
    struct Resolved {
        modules: Vec<String>,
        errors: Vec<String>,
    }

    #[test]
    fn resolve_modules() {
        let registry = Registry::new(&[
            ("app",    &["logging", "config", "ghost"]),
            ("logging", &["utils"]),
            ("config", &["utils"]),
            ("utils",  &[]),
            // "ghost" is not in the registry — will produce an error
        ]);

        // SeedGraph is a general anamorphism — three functions:
        // 1. seeds_from_node: a node's dependency names
        // 2. grow: dependency name → Either<Error, Module>
        // 3. seeds_from_top: entry point → initial names
        // seeds_for_fallible lifts Edgy<Module, String> to Edgy<Either<..>, String>:
        // valid modules produce seeds, errors produce none.
        let seeds_from_node = seeds_for_fallible(
            dom::edgy(move |module: &Module| module.deps.clone()),
        );
        let seed_graph = dom::SeedGraph::new(
            seeds_from_node,
            {
                let reg = registry;
                move |dep_name: &String| -> Either<ResolveError, Module> {
                    match reg.0.get(dep_name) {
                        Some(m) => Either::Right(m.clone()),
                        None => Either::Left(ResolveError(format!("not found: {}", dep_name))),
                    }
                }
            },
            dom::edgy(|top: &Vec<String>| top.clone()),
        );

        // The fold operates on Either<Error, Module> — both cases in one algebra.
        // Left (error): no children, init produces the error.
        // Right (valid): init produces the module name, children accumulate.
        let init = |node: &Either<ResolveError, Module>| match node {
            Either::Right(m) => Resolved {
                modules: vec![m.name.clone()],
                errors: vec![],
            },
            Either::Left(e) => Resolved {
                modules: vec![],
                errors: vec![e.0.clone()],
            },
        };
        let acc = |heap: &mut Resolved, child: &Resolved| {
            heap.modules.extend(child.modules.iter().cloned());
            heap.errors.extend(child.errors.iter().cloned());
        };
        let collect = dom::simple_fold(init, acc);

        // GraphWithFold wires graph + fold + a top-level heap initializer.
        // heap_of_top initializes the heap for the entry point (which isn't
        // a graph node — it's the spec that produces initial seeds).
        let graph = seed_graph.make_graph();
        let pipeline = GraphWithFold::new(
            &graph,
            &collect,
            |_top| Resolved { modules: vec![], errors: vec![] },
        );

        let top_deps = vec!["app".to_string()];
        let result = pipeline.run(&dom::FUSED, &top_deps);

        // Bottom-up order: utils resolved first, then logging, config, app
        // ghost produces an error
        assert!(result.modules.contains(&"utils".to_string()));
        assert!(result.modules.contains(&"app".to_string()));
        assert!(result.errors.contains(&"not found: ghost".to_string()));

        assert_snapshot!("resolution", format!(
            "resolved: [{}], errors: [{}]",
            result.modules.join(", "),
            result.errors.join(", "),
        ));
    }
}

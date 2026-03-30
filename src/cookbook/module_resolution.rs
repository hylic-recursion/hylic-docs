//! Minified module resolution — the pattern that motivated hylic.
//! Demonstrates: SeedGraph (entry point differs from recursion),
//! error handling via Either, and the "two-function pattern" solved.

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use either::Either;
    use hylic::fold::simple_fold;
    use hylic::graph::edgy;
    use hylic::ana::SeedGraph;
    use hylic::hylo::GraphWithFold;
    use hylic::cata::Exec;
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

        // SeedGraph solves the "entry point differs from recursion" pattern:
        // 1. seeds_from_valid: resolved module → dependency names (the recursive part)
        // 2. grow_node: dependency name → Either<Error, Module> (the lookup)
        // 3. seeds_from_top: top-level spec → initial names (the entry point)
        // From these three, hylic constructs a Treeish<Either<Error, Module>>.
        // Errors are Left nodes — they automatically have no children (no seeds).
        let seed_graph = SeedGraph::new(
            edgy(move |module: &Module| module.deps.clone()),
            {
                let reg = registry;
                move |dep_name: &String| -> Either<ResolveError, Module> {
                    match reg.0.get(dep_name) {
                        Some(m) => Either::Right(m.clone()),
                        None => Either::Left(ResolveError(format!("not found: {}", dep_name))),
                    }
                }
            },
            edgy(|top: &Vec<String>| top.clone()),
        );

        // The fold operates on Either<Error, Module> — both cases in one algebra.
        // Left (error): no children, init produces the error.
        // Right (valid): init produces the module name, children accumulate.
        let collect = simple_fold(
            |node: &Either<ResolveError, Module>| match node {
                Either::Right(m) => Resolved {
                    modules: vec![m.name.clone()],
                    errors: vec![],
                },
                Either::Left(e) => Resolved {
                    modules: vec![],
                    errors: vec![e.0.clone()],
                },
            },
            |heap: &mut Resolved, child: &Resolved| {
                heap.modules.extend(child.modules.iter().cloned());
                heap.errors.extend(child.errors.iter().cloned());
            },
        );

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
        let result = pipeline.run(&Exec::fused(), &top_deps);

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

//! Minified module resolution — the pattern that motivated hylic.
//! Demonstrates: SeedPipeline for lazy dependency discovery,
//! error handling via Either, and seeds_for_fallible.

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use either::Either;

    use hylic_pipeline::prelude::{SeedPipeline, PipelineExecSeed};
    use hylic::prelude::seeds_for_fallible;
    use hylic::domain::shared as dom;
    use hylic::graph;
    use insta::assert_snapshot;


    /// A module has a name and declares dependencies on other modules.
    #[derive(Clone, Debug)]
    struct Module {
        name: String,
        deps: Vec<String>,
    }

    /// A module registry: maps names to module definitions.
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

        // Node = Either<ResolveError, Module>.
        // seeds_from_node: valid modules produce dependency names (seeds),
        // errors produce none (via seeds_for_fallible).
        let seeds_from_node = seeds_for_fallible(
            graph::edgy(move |module: &Module| module.deps.clone()),
        );

        // grow: dependency name → Either<Error, Module>
        let grow = {
            let reg = registry;
            move |dep_name: &String| -> Either<ResolveError, Module> {
                match reg.0.get(dep_name) {
                    Some(m) => Either::Right(m.clone()),
                    None => Either::Left(ResolveError(format!("not found: {}", dep_name))),
                }
            }
        };

        // The fold operates on Either<Error, Module>.
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
        let collect = dom::fold(init, acc, |h| h.clone());

        // SeedPipeline: grow + seeds_from_node + fold.
        // Entry handled at the call site.
        let pipeline = SeedPipeline::new(grow, seeds_from_node, &collect);

        let result = pipeline.run_from_slice(
            &dom::FUSED,
            &["app".to_string()],
            Resolved { modules: vec![], errors: vec![] },
        );

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

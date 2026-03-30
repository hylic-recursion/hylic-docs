//! Filesystem tree summary — structured heap accumulating multiple metrics.

#[cfg(test)]
mod tests {
    use hylic::fold::simple_fold;
    use hylic::graph::treeish_visit;
    use hylic::cata::Strategy;
    use insta::assert_snapshot;

    // ANCHOR: filesystem_summary

    /// A filesystem entry: either a file (leaf) or a directory (branch).
    #[derive(Clone)]
    enum FsEntry {
        File { name: String, size: u64 },
        Dir { name: String, children: Vec<FsEntry> },
    }

    impl FsEntry {
        fn file(name: &str, size: u64) -> Self {
            FsEntry::File { name: name.into(), size }
        }
        fn dir(name: &str, ch: Vec<FsEntry>) -> Self {
            FsEntry::Dir { name: name.into(), children: ch }
        }
    }

    /// Accumulates size, file count, and directory count in one pass.
    #[derive(Clone, Debug, PartialEq)]
    struct Summary {
        total_size: u64,
        file_count: usize,
        dir_count: usize,
    }

    #[test]
    fn summarize_filesystem() {
        let tree = FsEntry::dir("project", vec![
            FsEntry::file("README.md", 1200),
            FsEntry::dir("src", vec![
                FsEntry::file("main.rs", 5000),
                FsEntry::file("lib.rs", 3000),
                FsEntry::dir("utils", vec![
                    FsEntry::file("helpers.rs", 800),
                ]),
            ]),
            FsEntry::file("Cargo.toml", 400),
        ]);

        // Tree structure: directories have children, files don't.
        let graph = treeish_visit(|entry: &FsEntry, cb: &mut dyn FnMut(&FsEntry)| {
            if let FsEntry::Dir { children, .. } = entry {
                for child in children { cb(child); }
            }
        });

        // Fold: each node initializes its own metric, children accumulate.
        let summarize = simple_fold(
            |entry: &FsEntry| match entry {
                FsEntry::File { size, .. } =>
                    Summary { total_size: *size, file_count: 1, dir_count: 0 },
                FsEntry::Dir { .. } =>
                    Summary { total_size: 0, file_count: 0, dir_count: 1 },
            },
            |heap: &mut Summary, child: &Summary| {
                heap.total_size += child.total_size;
                heap.file_count += child.file_count;
                heap.dir_count += child.dir_count;
            },
        );

        let result = Strategy::Sequential.run(&summarize, &graph, &tree);
        assert_eq!(result, Summary {
            total_size: 10400, file_count: 5, dir_count: 3,
        });

        // ANCHOR_END: filesystem_summary
        assert_snapshot!("fs_summary", format!(
            "project/: {} bytes, {} files, {} dirs",
            result.total_size, result.file_count, result.dir_count,
        ));
    }
}

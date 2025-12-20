// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::{BTreeSet, HashMap},
    path::PathBuf,
};

use cargo_metadata::{DependencyKind, Metadata, MetadataCommand, Package};
use petgraph::{algo::all_simple_paths, prelude::DiGraphMap};

type PackageGraph<'p> = DiGraphMap<&'p str, ()>;

struct Packages(HashMap<String, Package>);

#[test]
/// Make sure that all accesses to execution layer crates in the `rtd-node` and `rtd-replay` crates
/// go via the `rtd-execution` crate (in other words, the `rtd-execution` crate dominates execution
/// layer crates in the dependency graphs of `rtd-node` and `rtd-replay`).
///
/// This helps ensures that execution that may be committed on-chain respects the execution version
/// that is stated in the protocol config.
fn test_encapsulation() {
    let metadata = cargo_metadata().unwrap();
    let packages = Packages::new(&metadata);

    // Identify the crates that are part of the execution layer
    let mut exec_crates: BTreeSet<_> = packages.normal_deps("rtd-execution").collect();

    // Remove the crates that the execution layer depends on but which are not directly part of the
    // execution layer -- these don't need to be accessed exclusively via `rtd-execution`.
    exec_crates.remove("rtd-protocol-config");
    exec_crates.remove("rtd-types");
    exec_crates.remove("move-binary-format");
    exec_crates.remove("move-bytecode-utils");
    exec_crates.remove("move-core-types");
    exec_crates.remove("move-vm-config");
    // tracing is only enabled in client builds (built with `--features tracing` flag)
    // and it does not have to be accessed via `rtd-execution` as it can never cause a fork
    exec_crates.remove("move-trace-format");

    // Capture problematic paths from roots to execution crates
    let mut examples = vec![];

    for root in ["rtd-node", "rtd-replay"] {
        let mut graph = packages.graph(root);

        // If we can still create a path from `root` to an execution crate after removing these
        // nodes then we know that we can potential bypass "rtd-execution".
        graph.remove_node("rtd-execution");

        for exec_crate in &exec_crates {
            let paths = all_simple_paths::<Vec<&str>, &PackageGraph>(
                &graph, root, exec_crate, /* min_intermediate_nodes */ 0,
                /* max_intermediate_nodes */ None,
            );

            examples.extend(paths.map(|p| p.join(" -> ")));
        }
    }

    if examples.is_empty() {
        return;
    }

    panic!(
        "protocol-sensitive binaries depend on execution crates outside of 'rtd-execution', e.g.:\n\
         \n  {}\n\
         \n\
         This can cause execution to fork by not respecting the execution layer version set in the \
         protocol config.  Fix this by depending on these crates via 'rtd-execution'.\n\
         \n\
         P.S. if you believe one of these crates should not be part of 'rtd-execution' then update \
         the test to exclude this crate.",
        examples.join("\n  "),
    );
}

/// Parse `cargo metadata` for the `rtd` repo.
fn cargo_metadata() -> cargo_metadata::Result<Metadata> {
    let rtd_execution = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    MetadataCommand::new()
        .manifest_path(rtd_execution.join("../Cargo.toml"))
        .current_dir(rtd_execution.join(".."))
        .no_deps()
        .exec()
}

impl Packages {
    /// Create a mapping from package names to package `metadata` (from the output of `cargo
    /// metadata`).
    fn new(metadata: &Metadata) -> Self {
        Self(HashMap::from_iter(
            metadata
                .packages
                .iter()
                .map(|pkg| (pkg.name.clone(), pkg.clone())),
        ))
    }

    /// Extract the transitive dependency sub-graph of the package named `root`.  The graph is a
    /// directed, unweighted graph with nodes representing packages, identified by their name (a
    /// `&str`).
    fn graph<'p>(&'p self, root: &'p str) -> PackageGraph<'p> {
        let mut graph = PackageGraph::new();
        let mut stack = vec![];

        stack.extend(self.normal_edges(root));
        while let Some((from, to)) = stack.pop() {
            if !graph.contains_node(to) {
                graph.add_edge(from, to, ());
                stack.extend(self.normal_edges(to))
            }
        }

        graph
    }

    /// Returns an iterator over all the edges from `pkg` to its "normal" dependencies (represented
    /// as pairs of Node IDs).  A normal dependency is a non-target specific, non-build, non-dev
    /// dependency.
    fn normal_edges<'p, 'q>(&'q self, pkg: &'p str) -> impl Iterator<Item = (&'p str, &'q str)> {
        self.0
            .get(pkg)
            .map(|p| &p.dependencies)
            .into_iter()
            .flatten()
            .filter_map(move |dep| {
                if let (DependencyKind::Normal, None) = (dep.kind, &dep.target) {
                    Some((pkg, dep.name.as_str()))
                } else {
                    None
                }
            })
    }

    /// Returns an iterator over all of `pkg`'s "normal" dependencies. (See [normal_edges] for a
    /// definition of "normal").
    fn normal_deps<'p, 'q: 'p>(&'q self, pkg: &'p str) -> impl 'p + Iterator<Item = &'q str> {
        self.normal_edges(pkg).map(move |(_, to)| to)
    }
}

//! End-to-end check against a real language server (rust-analyzer).
//!
//! Ignored by default: CI does not guarantee a language server. Run with
//! `cargo test --lib semantic::e2e_tests -- --ignored` on a machine with
//! `rustup component add rust-analyzer`.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::core::call_graph::{
    CallGraph, CallGraphInputs, StructuralTarget, SymbolSpan, resolve_edge_callee_targets,
};
use crate::core::config::SemanticMode;
use crate::core::property_graph::CodeGraph;

use super::implementations::resolve_implements_edges;
use super::{SemanticVerdict, escalate_calls};

const FILES: &[(&str, &str)] = &[
    (
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    ),
    (
        "src/lib.rs",
        "pub mod a;\npub mod app;\npub mod b;\npub mod mem;\npub mod pg;\npub mod store;\n",
    ),
    (
        "src/a.rs",
        "pub struct UserRepo;\nimpl UserRepo {\n    pub fn save(&self) {}\n}\n",
    ),
    (
        "src/b.rs",
        "pub struct PaymentRepo;\nimpl PaymentRepo {\n    pub fn save(&self) {}\n}\n",
    ),
    (
        "src/app.rs",
        "pub fn checkout(repo: &crate::b::PaymentRepo) {\n    repo.save();\n}\n",
    ),
    ("src/store.rs", "pub trait Store {\n    fn put(&self);\n}\n"),
    (
        "src/pg.rs",
        "pub struct Pg;\nimpl crate::store::Store for Pg {\n    fn put(&self) {}\n}\n",
    ),
    (
        "src/mem.rs",
        "pub struct Mem;\nimpl crate::store::Store for Mem {\n    fn put(&self) {}\n}\n",
    ),
];

fn span(file: &str, name: &str, start: usize, end: usize, kind: &str) -> SymbolSpan {
    SymbolSpan {
        file: file.into(),
        name: name.into(),
        start_line: start,
        end_line: end,
        kind: kind.into(),
    }
}

#[test]
#[ignore = "needs rust-analyzer (rustup component add rust-analyzer)"]
fn rust_analyzer_verifies_ambiguous_calls_and_trait_implementations() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    for (path, body) in FILES {
        let p = root.join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }
    let root_str = root.to_string_lossy().to_string();
    let inputs = CallGraphInputs {
        project_root: root_str.clone(),
        file_paths: FILES[1..].iter().map(|(p, _)| (*p).to_string()).collect(),
        symbols: vec![
            span("src/a.rs", "save", 3, 3, "method"),
            span("src/b.rs", "save", 3, 3, "method"),
            span("src/app.rs", "checkout", 1, 3, "fn"),
            span("src/store.rs", "Store", 1, 3, "trait"),
        ],
        import_edges: Vec::new(),
    };

    let call_graph = CallGraph::build(&inputs);
    let calls: Vec<_> = call_graph
        .edges
        .iter()
        .filter(|e| e.callee_name == "save")
        .cloned()
        .collect();
    assert_eq!(calls.len(), 1, "one call site: {:?}", call_graph.edges);
    let structural = resolve_edge_callee_targets(&inputs, &calls);
    assert_eq!(
        structural,
        vec![StructuralTarget::Ambiguous],
        "two `save`s, no import"
    );

    let graph = CodeGraph::open_in_memory().unwrap();
    let hashes: &HashMap<String, String> = &call_graph.file_hashes;

    // A cold server answers "no definition" while indexing; retry until it
    // has loaded the workspace (bounded).
    let deadline = Instant::now() + Duration::from_mins(3);
    let verdict = loop {
        let verdicts = escalate_calls(
            &graph,
            &root_str,
            &inputs,
            &calls,
            &structural,
            hashes,
            SemanticMode::Eager,
            super::EscalationBudget::BACKGROUND,
        )
        .verdicts;
        if verdicts[0].is_some() || Instant::now() > deadline {
            break verdicts[0].clone();
        }
        std::thread::sleep(Duration::from_secs(2));
    };
    match verdict {
        Some(SemanticVerdict::Verified { file, backend }) => {
            assert_eq!(file, "src/b.rs", "repo is a PaymentRepo");
            assert!(backend.starts_with("lsp:rust-analyzer"), "{backend}");
        }
        other => panic!("expected a verified call to src/b.rs, got {other:?}"),
    }

    // Second run: answered from the cache, no live query.
    let stats = escalate_calls(
        &graph,
        &root_str,
        &inputs,
        &calls,
        &structural,
        hashes,
        SemanticMode::Eager,
        super::EscalationBudget::BACKGROUND,
    )
    .stats;
    assert_eq!((stats.cache_hits, stats.live_queries), (1, 0));

    // Implementations: a cold answer is empty and leaves the file unsettled.
    let pass = loop {
        let pass =
            resolve_implements_edges(&graph, &root_str, &inputs, hashes, SemanticMode::Eager);
        if !pass.edges.is_empty() || Instant::now() > deadline {
            break pass;
        }
        std::thread::sleep(Duration::from_secs(2));
    };
    let implementors: Vec<String> = pass
        .edges
        .into_iter()
        .map(|e| {
            assert_eq!(e.abstract_file, "src/store.rs");
            e.impl_file
        })
        .collect();
    assert_eq!(
        implementors,
        vec!["src/mem.rs".to_string(), "src/pg.rs".to_string()]
    );
    assert!(pass.settled.contains("src/store.rs"));

    crate::lsp::router::shutdown_all();
}

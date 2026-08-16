//! End-to-end coverage for the Solution Intelligence `ctx_optimize` surface.

use std::path::PathBuf;
use std::process::Command;

use lean_ctx::core::solution_auto_capture::{detect_debt_marker, detect_stdlib_choice};
use serde_json::{Value, json};

struct Sandbox {
    _root: tempfile::TempDir,
    home: PathBuf,
    config: PathBuf,
    data: PathBuf,
    state: PathBuf,
    cache: PathBuf,
    project: PathBuf,
}

impl Sandbox {
    fn new() -> Self {
        let root = tempfile::tempdir().expect("create solution-intelligence sandbox");
        let home = root.path().join("home");
        let config = root.path().join("config");
        let data = root.path().join("data");
        let state = root.path().join("state");
        let cache = root.path().join("cache");
        let project = root.path().join("project");
        for dir in [&home, &config, &data, &state, &cache, &project] {
            std::fs::create_dir_all(dir).expect("create sandbox directory");
        }
        std::fs::write(
            config.join("config.toml"),
            "[solution]\nintensity = \"Balanced\"\n\n[solution.commercial.team_policy]\nenabled = true\nmin_intensity = \"balanced\"\n",
        )
        .expect("write isolated solution configuration");

        Self {
            _root: root,
            home,
            config,
            data,
            state,
            cache,
            project,
        }
    }

    fn optimize(&self, args: &Value) -> String {
        let output = Command::new(env!("CARGO_BIN_EXE_lean-ctx"))
            .args([
                "call",
                "ctx_optimize",
                "--project-root",
                self.project.to_str().expect("UTF-8 project root"),
                "--json",
                &args.to_string(),
            ])
            .env_clear()
            .env("HOME", &self.home)
            .env("LEAN_CTX_CONFIG_DIR", &self.config)
            .env("LEAN_CTX_DATA_DIR", &self.data)
            .env("LEAN_CTX_STATE_DIR", &self.state)
            .env("LEAN_CTX_CACHE_DIR", &self.cache)
            .env("LEAN_CTX_DISABLED", "1")
            .output()
            .expect("run ctx_optimize through the CLI registry");

        assert!(
            output.status.success(),
            "ctx_optimize failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).expect("ctx_optimize must return UTF-8")
    }
}

#[test]
fn solution_intelligence_actions_persist_across_real_tool_calls() {
    let sandbox = Sandbox::new();

    let ladder = sandbox.optimize(&json!({ "action": "ladder" }));
    assert!(ladder.contains("Solution efficiency ladder:"));
    assert!(ladder.contains("3. Stdlib:"));
    assert!(ladder.contains("7. Minimum:"));

    let decision = sandbox.optimize(&json!({
        "action": "decide",
        "category": "stdlib",
        "decision": "Use HashMap from the standard library"
    }));
    assert_eq!(
        decision.trim(),
        "Recorded stdlib decision: Use HashMap from the standard library"
    );

    let tracker_path = sandbox.data.join("solution_tracker.json");
    let tracker: Value = serde_json::from_slice(
        &std::fs::read(&tracker_path).expect("decision must persist tracker state"),
    )
    .expect("persisted tracker must be JSON");
    assert_eq!(tracker["decisions_total"], 1);
    assert_eq!(tracker["decisions_stdlib"], 1);

    let report = sandbox.optimize(&json!({ "action": "report" }));
    assert!(
        report.contains("1 decisions; 0 net LOC saved; 0% output-token reduction"),
        "unexpected report: {report}"
    );

    let fingerprint = sandbox.optimize(&json!({
        "action": "fingerprint",
        "decision": "Refactor the request pipeline"
    }));
    assert!(fingerprint.contains("Prediction:"));
    assert!(fingerprint.contains("rung='reuse'"));
    assert!(fingerprint.contains("pattern='refactor'"));

    let policy = sandbox.optimize(&json!({ "action": "policy-check" }));
    assert!(
        policy.contains("Policy check passed. Intensity 'balanced' is compliant."),
        "unexpected policy result: {policy}"
    );
}

#[test]
fn auto_capture_detects_a_new_standard_library_choice() {
    let old_content = "use crate::core::Config;\n";
    let new_content = "use crate::core::Config;\nuse std::collections::HashMap;\n";

    assert_eq!(
        detect_stdlib_choice(old_content, new_content),
        Some("Selected standard library import `use std::collections::HashMap;`".to_string())
    );
}

#[test]
fn auto_capture_detects_lean_ctx_debt_markers() {
    assert_eq!(
        detect_debt_marker("  // lean-ctx: remove after migration\n"),
        Some("Added lean-ctx debt marker: remove after migration".to_string())
    );
}

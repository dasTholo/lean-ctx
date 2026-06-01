//! Phase-1 gate (spec §6): `@lean-md` header + `@include` + an `@read` R-router
//! render end-to-end through the native lmd rushdown extension.

use lean_ctx::lmd::engine::render;

#[test]
fn gate_header_include_read_render_e2e() {
    let f = std::env::temp_dir().join("lmd_gate_fixture.txt");
    std::fs::write(&f, "GATE_FIXTURE_CONTENT\n").unwrap();

    let input = format!(
        "@lean-md 0.1\nconsumer: ai\n\n@include hard-rules\n\n@read {}\n",
        f.to_str().unwrap()
    );
    let out = render(&input);

    // header consumed
    assert!(!out.contains("@lean-md"), "header leaked: {out}");
    // @include rendered the built-in hard-rules fragment
    assert!(out.contains("lean-ctx"), "include did not render: {out}");
    // @read routed into ctx_read and surfaced the file content
    assert!(out.contains("GATE_FIXTURE_CONTENT"), "read did not render: {out}");
}

#[test]
fn gate_plain_markdown_passes_through() {
    let out = render("# Title\n\nsome **bold** text\n");
    assert!(out.contains("Title"));
    assert!(out.contains("<strong>bold</strong>"), "commonmark still works: {out}");
}

#[test]
fn gate_reread_warms_shared_cache_without_fresh() {
    // Read→Delta (spec §4.2a): repeated `@read … mode=full` of the SAME path in ONE
    // render share the EngineContext cache, so a later re-read collapses to an
    // `[unchanged …]` cache-hit stub WITHOUT any fresh/raw. The stub is a mode=full
    // feature (ctx_read.rs:713-743). Proving the stub appears proves the shared cache
    // warms. (Auto mode does not surface the stub — a separate ctx_read follow-up.)
    let f = std::env::temp_dir().join("lmd_gate_reread.txt");
    std::fs::write(&f, "GATE_REREAD_SENTINEL\n").unwrap();
    let p = f.to_str().unwrap();
    let out = render(&format!(
        "@read {p} mode=full\n\n@read {p} mode=full\n\n@read {p} mode=full\n"
    ));
    assert!(
        out.contains("[unchanged"),
        "a mode=full re-read must warm the shared cache into an `[unchanged` stub (no fresh/raw); got: {out}"
    );
}

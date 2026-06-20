//! Deterministic agent-bench scenarios against the checked-in fixture repo.
//! These run in CI without an LLM — they validate verify, context, sandbox, and diff plumbing.

mod live_eval;

use std::path::{Path, PathBuf};

use crate::changes::ChangeTracker;
use crate::context::build_context_pack;
use crate::tools::{
    execute_tool, infer_verify_command, resolve_verify_command, run_verify, tool_context_from_settings,
    WorkspaceSandbox,
};

pub fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rust-calc")
}

#[test]
fn fixture_infers_cargo_test() {
    let root = fixture_root();
    assert!(root.join("Cargo.toml").exists());
    assert_eq!(
        infer_verify_command(&root).as_deref(),
        Some("cargo test")
    );
    assert_eq!(
        resolve_verify_command(&root, None).as_deref(),
        Some("cargo test")
    );
}

#[test]
fn fixture_context_pack_lists_crate_layout() {
    let pack = build_context_pack(&fixture_root());
    assert!(pack.contains("Cargo.toml"));
    assert!(pack.contains("src"));
}

#[test]
fn fixture_edit_tracks_unified_diff() {
    let ctx = tool_context_from_settings(
        &Some(fixture_root().to_string_lossy().into()),
        true,
        &[],
    )
        .expect("fixture context");
    let path = "src/lib.rs";
    let before = std::fs::read_to_string(ctx.sandbox.resolve(path).unwrap()).unwrap();

    let mut tracker = ChangeTracker::default();
    tracker.capture_before(&ctx.sandbox, path);

    let result = execute_tool(
        &ctx,
        "edit_file",
        &serde_json::json!({
            "path": path,
            "old_string": "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}",
            "new_string": "pub fn add(a: i32, b: i32) -> i32 {\n    a + b + 1\n}",
        }),
    );
    assert!(result.ok, "{}", result.error.unwrap_or_default());

    tracker.note_touched(path);
    let changes = tracker.finalize(&ctx.sandbox);
    assert_eq!(changes.len(), 1);
    assert!(changes[0].diff.contains('+'));
    assert!(changes[0].diff.contains('-'));

    std::fs::write(ctx.sandbox.resolve(path).unwrap(), before).unwrap();
}

#[test]
fn fixture_verify_command_runs_in_sandbox() {
    let ctx = tool_context_from_settings(
        &Some(fixture_root().to_string_lossy().into()),
        true,
        &[],
    )
        .expect("fixture context");
    let result = run_verify(&ctx.sandbox, "cargo test", &[]);
    assert!(result.ok, "{}", result.output);
}

#[test]
fn fixture_sandbox_blocks_escape() {
    let ctx = tool_context_from_settings(
        &Some(fixture_root().to_string_lossy().into()),
        true,
        &[],
    )
        .expect("fixture context");
    assert!(ctx.sandbox.resolve("../Cargo.toml").is_err());
}

#[test]
fn fixture_read_and_grep_find_symbol() {
    let ctx = tool_context_from_settings(
        &Some(fixture_root().to_string_lossy().into()),
        true,
        &[],
    )
        .expect("fixture context");

    let content = execute_tool(
        &ctx,
        "read_file",
        &serde_json::json!({ "path": "src/lib.rs" }),
    );
    assert!(content.ok);
    assert!(content.output.contains("add"));

    let matches = execute_tool(
        &ctx,
        "grep",
        &serde_json::json!({ "pattern": "subtract", "path": "src" }),
    );
    assert!(matches.ok);
    assert!(matches.output.contains("subtract"));
}

pub fn scenario_manifest() -> &'static [(&'static str, fn(&Path) -> bool)] {
    &[
        (
            "infer_verify_command",
            |root| infer_verify_command(root).is_some(),
        ),
        (
            "context_pack",
            |root| build_context_pack(root).contains("Cargo.toml"),
        ),
        (
            "sandbox_root",
            |root| WorkspaceSandbox::from_root(root).is_ok(),
        ),
    ]
}

#[test]
fn fixture_scenario_manifest_passes() {
    let root = fixture_root();
    for (name, check) in scenario_manifest() {
        assert!(check(&root), "scenario failed: {name}");
    }
}

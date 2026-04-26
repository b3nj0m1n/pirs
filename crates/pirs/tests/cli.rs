//! CLI integration tests for the `pirs` binary.
//!
//! Covers the headline acceptance criteria from `spec/pirs_requirements_spec.md`.

use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

fn pirs() -> Command {
    Command::cargo_bin("pirs").expect("pirs binary built")
}

// ---------------------------------------------------------------------------
// Help / version
// ---------------------------------------------------------------------------

#[test]
fn shows_help() {
    pirs()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Post-Incident Review"));
}

// ---------------------------------------------------------------------------
// AC-001: pirs init
// ---------------------------------------------------------------------------

#[test]
fn ac_001_init_creates_pir_dir_without_sample_pir() {
    let temp = assert_fs::TempDir::new().unwrap();
    pirs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    temp.child("pirs.toml").assert(predicate::path::exists());
    temp.child("doc/pir").assert(predicate::path::is_dir());

    // No fake/sample PIR is created.
    let entries: Vec<_> = std::fs::read_dir(temp.child("doc/pir").path())
        .unwrap()
        .collect();
    assert!(
        entries.is_empty(),
        "init must not create a sample PIR; found {entries:?}"
    );
}

// ---------------------------------------------------------------------------
// AC-002: agent-only development incident
// ---------------------------------------------------------------------------

#[test]
fn ac_002_agent_only_development_incident() {
    let temp = assert_fs::TempDir::new().unwrap();
    pirs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    pirs()
        .current_dir(temp.path())
        .args([
            "new",
            "Failing cargo test after parser change",
            "--type",
            "development",
            "--severity",
            "medium",
            "--agent",
            "GitHub Copilot",
            "--problem",
            "cargo test failed after parser metadata update",
            "--no-edit",
        ])
        .assert()
        .success();

    let f = temp.child("doc/pir/0001-failing-cargo-test-after-parser-change.md");
    f.assert(predicate::path::exists());
    f.assert(predicate::str::contains("GitHub Copilot"));
    f.assert(predicate::str::contains("Open"));
    f.assert(predicate::str::contains("type: agent"));
    f.assert(predicate::str::contains("type: detected"));
}

// ---------------------------------------------------------------------------
// AC-003 partial: pirs run --on-fail create
// ---------------------------------------------------------------------------

#[test]
fn ac_003_run_on_fail_creates_pir_and_propagates_exit_code() {
    let temp = assert_fs::TempDir::new().unwrap();
    pirs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    pirs()
        .current_dir(temp.path())
        .args([
            "run",
            "--on-fail",
            "create",
            "--agent",
            "GitHub Copilot",
            "--",
            "sh",
            "-c",
            "exit 7",
        ])
        .assert()
        .code(7);

    let dir = temp.child("doc/pir");
    let mut entries: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap())
        .collect();
    entries.retain(|e| {
        e.path()
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with(".md"))
    });
    assert_eq!(entries.len(), 1, "expected exactly one PIR file");
    let body = std::fs::read_to_string(entries[0].path()).unwrap();
    assert!(body.contains("exit_code: 7"), "body: {body}");
    assert!(body.contains("agent-command-runner"));
}

// ---------------------------------------------------------------------------
// AC-007: review gate prevents premature closure
// ---------------------------------------------------------------------------

#[test]
fn ac_007_review_gate_blocks_when_incomplete() {
    let temp = assert_fs::TempDir::new().unwrap();
    pirs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();
    pirs()
        .current_dir(temp.path())
        .args([
            "new",
            "Tests broke",
            "--problem",
            "auth tests fail",
            "--no-edit",
        ])
        .assert()
        .success();

    pirs()
        .current_dir(temp.path())
        .args(["status", "1", "reviewed"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not ready for Reviewed"));
}

// ---------------------------------------------------------------------------
// AC-005 / AC-006: 5 Whys + actions add via CLI
// ---------------------------------------------------------------------------

#[test]
fn ac_005_006_why_and_action_add() {
    let temp = assert_fs::TempDir::new().unwrap();
    pirs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();
    pirs()
        .current_dir(temp.path())
        .args(["new", "x", "--problem", "broken", "--no-edit"])
        .assert()
        .success();

    pirs()
        .current_dir(temp.path())
        .args([
            "why",
            "add",
            "1",
            "--question",
            "why did it break?",
            "--answer",
            "config drifted",
        ])
        .assert()
        .success();

    pirs()
        .current_dir(temp.path())
        .args([
            "action",
            "add",
            "1",
            "--description",
            "Add regression test",
            "--owner",
            "GitHub Copilot",
            "--owner-type",
            "agent",
            "--due",
            "2026-12-31",
        ])
        .assert()
        .success();

    pirs()
        .current_dir(temp.path())
        .args(["actions", "--owner", "GitHub Copilot"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ACT-001"))
        .stdout(predicate::str::contains("Add regression test"));
}

// ---------------------------------------------------------------------------
// AC-009: search across body fields
// ---------------------------------------------------------------------------

#[test]
fn ac_009_search_finds_problem_statement_text() {
    let temp = assert_fs::TempDir::new().unwrap();
    pirs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();
    pirs()
        .current_dir(temp.path())
        .args([
            "new",
            "Checkout retry",
            "--problem",
            "retry state lost during refactor",
            "--no-edit",
        ])
        .assert()
        .success();

    pirs()
        .current_dir(temp.path())
        .args(["search", "retry state"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Checkout retry"));
}

// ---------------------------------------------------------------------------
// AC-010 / AC-011: doctor + JSON export
// ---------------------------------------------------------------------------

#[test]
fn ac_010_doctor_reports_clean_repo() {
    let temp = assert_fs::TempDir::new().unwrap();
    pirs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();
    pirs()
        .current_dir(temp.path())
        .args(["new", "x", "--problem", "y", "--no-edit"])
        .assert()
        .success();
    pirs()
        .current_dir(temp.path())
        .arg("doctor")
        .assert()
        .success();
}

#[test]
fn ac_011_export_json_emits_schema_and_pir() {
    let temp = assert_fs::TempDir::new().unwrap();
    pirs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();
    pirs()
        .current_dir(temp.path())
        .args(["new", "Foo", "--problem", "bar", "--no-edit"])
        .assert()
        .success();

    let assert = pirs()
        .current_dir(temp.path())
        .args(["export", "json"])
        .assert()
        .success();
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(out.contains("\"schema\""));
    assert!(out.contains("\"pirs\""));
    assert!(out.contains("Foo"));
}

// ---------------------------------------------------------------------------
// REQ-TIME-003: durations recomputed when status moves to resolved
// ---------------------------------------------------------------------------

#[test]
fn status_resolved_now_sets_resolved_at_and_duration() {
    let temp = assert_fs::TempDir::new().unwrap();
    pirs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();
    pirs()
        .current_dir(temp.path())
        .args(["new", "x", "--problem", "y", "--no-edit"])
        .assert()
        .success();

    pirs()
        .current_dir(temp.path())
        .args(["status", "1", "resolved", "--now"])
        .assert()
        .success();

    let body = std::fs::read_to_string(temp.child("doc/pir/0001-x.md").path()).unwrap();
    assert!(body.contains("resolved_at:"));
    assert!(body.contains("status: Resolved"));
    assert!(body.contains("time_to_resolve:"));
}

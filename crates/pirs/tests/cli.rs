//! CLI integration tests for the `pirs` binary.
//!
//! Covers the headline acceptance criteria from `spec/pirs_requirements_spec.md`.

use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

fn pirs() -> Command {
    Command::cargo_bin("pirs").expect("pirs binary built")
}

fn configure_redaction(temp: &assert_fs::TempDir) {
    temp.child("pirs.toml")
        .write_str(
            r#"pir_dir = "doc/pir"

[templates]

[privacy]
redaction_patterns = ["token=[A-Za-z0-9_-]+"]
sensitive_fields = []

[mcp]
http_enabled = false
"#,
        )
        .unwrap();
}

fn export_json(temp: &assert_fs::TempDir) -> String {
    let assert = pirs()
        .current_dir(temp.path())
        .args(["export", "json"])
        .assert()
        .success();
    String::from_utf8(assert.get_output().stdout.clone()).unwrap()
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

#[test]
fn ac_011_export_json_redact_masks_configured_patterns() {
    let temp = assert_fs::TempDir::new().unwrap();
    pirs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();
    configure_redaction(&temp);
    pirs()
        .current_dir(temp.path())
        .args([
            "new",
            "Leaky command output",
            "--problem",
            "captured stdout contained token=abc123 and a harmless value",
            "--no-edit",
        ])
        .assert()
        .success();

    let plain = export_json(&temp);
    assert!(plain.contains("token=abc123"), "plain export: {plain}");

    let assert = pirs()
        .current_dir(temp.path())
        .args(["export", "json", "--redact"])
        .assert()
        .success();
    let redacted = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(
        !redacted.contains("token=abc123"),
        "redacted export: {redacted}"
    );
    assert!(
        redacted.contains("[REDACTED]"),
        "redacted export: {redacted}"
    );
}

#[test]
fn import_json_file_creates_pir_from_bulk_export() {
    let source = assert_fs::TempDir::new().unwrap();
    pirs()
        .current_dir(source.path())
        .arg("init")
        .assert()
        .success();
    pirs()
        .current_dir(source.path())
        .args([
            "new",
            "Round trip incident",
            "--problem",
            "exported from another repository",
            "--no-edit",
        ])
        .assert()
        .success();
    let exported = export_json(&source);

    let target = assert_fs::TempDir::new().unwrap();
    pirs()
        .current_dir(target.path())
        .arg("init")
        .assert()
        .success();
    let import_file = target.child("import.json");
    import_file.write_str(&exported).unwrap();

    pirs()
        .current_dir(target.path())
        .args(["import", "json", import_file.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("NEW 0001 Round trip incident"))
        .stdout(predicate::str::contains("imported 1"));

    let imported = target.child("doc/pir/0001-round-trip-incident.md");
    imported.assert(predicate::path::exists());
    imported.assert(predicate::str::contains("exported from another repository"));
}

#[test]
fn import_json_stdin_dry_run_reports_without_writing() {
    let source = assert_fs::TempDir::new().unwrap();
    pirs()
        .current_dir(source.path())
        .arg("init")
        .assert()
        .success();
    pirs()
        .current_dir(source.path())
        .args([
            "new",
            "Dry run incident",
            "--problem",
            "should not be written",
            "--no-edit",
        ])
        .assert()
        .success();
    let exported = export_json(&source);

    let target = assert_fs::TempDir::new().unwrap();
    pirs()
        .current_dir(target.path())
        .arg("init")
        .assert()
        .success();

    pirs()
        .current_dir(target.path())
        .args(["import", "json", "-", "--dry-run"])
        .write_stdin(exported)
        .assert()
        .success()
        .stdout(predicate::str::contains("NEW 0001 Dry run incident"))
        .stdout(predicate::str::contains("dry run"));

    let entries: Vec<_> = std::fs::read_dir(target.child("doc/pir").path())
        .unwrap()
        .collect();
    assert!(
        entries.is_empty(),
        "dry-run must not create files: {entries:?}"
    );
}

#[test]
fn import_json_skips_existing_number_unless_overwrite_is_supplied() {
    let source = assert_fs::TempDir::new().unwrap();
    pirs()
        .current_dir(source.path())
        .arg("init")
        .assert()
        .success();
    pirs()
        .current_dir(source.path())
        .args([
            "new",
            "Source incident",
            "--problem",
            "source problem",
            "--no-edit",
        ])
        .assert()
        .success();
    let exported = export_json(&source);

    let target = assert_fs::TempDir::new().unwrap();
    pirs()
        .current_dir(target.path())
        .arg("init")
        .assert()
        .success();
    pirs()
        .current_dir(target.path())
        .args([
            "new",
            "Existing incident",
            "--problem",
            "existing problem",
            "--no-edit",
        ])
        .assert()
        .success();
    let import_file = target.child("import.json");
    import_file.write_str(&exported).unwrap();

    pirs()
        .current_dir(target.path())
        .args(["import", "json", import_file.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("SKIP 0001 Source incident"))
        .stdout(predicate::str::contains("imported 0"));
    target
        .child("doc/pir/0001-existing-incident.md")
        .assert(predicate::str::contains("existing problem"));
    target
        .child("doc/pir/0001-source-incident.md")
        .assert(predicate::path::missing());

    pirs()
        .current_dir(target.path())
        .args([
            "import",
            "json",
            import_file.path().to_str().unwrap(),
            "--overwrite",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("OVERWRITE 0001 Source incident"))
        .stdout(predicate::str::contains("imported 1"));
    target
        .child("doc/pir/0001-existing-incident.md")
        .assert(predicate::path::missing());
    target
        .child("doc/pir/0001-source-incident.md")
        .assert(predicate::str::contains("source problem"));
}

#[test]
fn import_json_overwrite_removes_all_existing_number_files() {
    let source = assert_fs::TempDir::new().unwrap();
    pirs()
        .current_dir(source.path())
        .arg("init")
        .assert()
        .success();
    pirs()
        .current_dir(source.path())
        .args([
            "new",
            "Source incident",
            "--problem",
            "source problem",
            "--no-edit",
        ])
        .assert()
        .success();
    let exported = export_json(&source);

    let target = assert_fs::TempDir::new().unwrap();
    pirs()
        .current_dir(target.path())
        .arg("init")
        .assert()
        .success();
    target
        .child("doc/pir/0001-first.md")
        .write_str("first")
        .unwrap();
    target
        .child("doc/pir/0001-second.md")
        .write_str("second")
        .unwrap();
    let import_file = target.child("import.json");
    import_file.write_str(&exported).unwrap();

    pirs()
        .current_dir(target.path())
        .args([
            "import",
            "json",
            import_file.path().to_str().unwrap(),
            "--overwrite",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("OVERWRITE 0001 Source incident"))
        .stdout(predicate::str::contains("overwritten 1"));

    target
        .child("doc/pir/0001-first.md")
        .assert(predicate::path::missing());
    target
        .child("doc/pir/0001-second.md")
        .assert(predicate::path::missing());
    target
        .child("doc/pir/0001-source-incident.md")
        .assert(predicate::str::contains("source problem"));
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

// ---------------------------------------------------------------------------
// REQ-RPT-001: pirs generate report <PIR>
// ---------------------------------------------------------------------------

#[test]
fn generate_report_renders_required_sections() {
    let temp = assert_fs::TempDir::new().unwrap();
    pirs().current_dir(temp.path()).arg("init").assert().success();
    pirs()
        .current_dir(temp.path())
        .args([
            "new",
            "Failing build",
            "--problem",
            "cargo build failed",
            "--severity",
            "high",
            "--no-edit",
        ])
        .assert()
        .success();

    pirs()
        .current_dir(temp.path())
        .args(["generate", "report", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# PIR-0001: Failing build"))
        .stdout(predicate::str::contains("## Problem Statement"))
        .stdout(predicate::str::contains("cargo build failed"));
}

// ---------------------------------------------------------------------------
// REQ-RPT-002: pirs generate actions
// ---------------------------------------------------------------------------

#[test]
fn generate_actions_lists_open_actions_first() {
    let temp = assert_fs::TempDir::new().unwrap();
    pirs().current_dir(temp.path()).arg("init").assert().success();
    pirs()
        .current_dir(temp.path())
        .args(["new", "First", "--problem", "p", "--no-edit"])
        .assert()
        .success();
    pirs()
        .current_dir(temp.path())
        .args([
            "action", "add", "1", "--description", "fix it", "--owner", "alice",
        ])
        .assert()
        .success();

    pirs()
        .current_dir(temp.path())
        .args(["generate", "actions"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Action Register"))
        .stdout(predicate::str::contains("ACT-001"))
        .stdout(predicate::str::contains("alice"));
}

#[test]
fn generate_actions_handles_empty_repository() {
    let temp = assert_fs::TempDir::new().unwrap();
    pirs().current_dir(temp.path()).arg("init").assert().success();
    pirs()
        .current_dir(temp.path())
        .args(["generate", "actions"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No actions recorded"));
}

// ---------------------------------------------------------------------------
// REQ-RPT-003: pirs metrics
// ---------------------------------------------------------------------------

#[test]
fn metrics_summarizes_counts_and_open_actions() {
    let temp = assert_fs::TempDir::new().unwrap();
    pirs().current_dir(temp.path()).arg("init").assert().success();
    pirs()
        .current_dir(temp.path())
        .args([
            "new", "p1", "--problem", "x", "--severity", "high", "--no-edit",
        ])
        .assert()
        .success();
    pirs()
        .current_dir(temp.path())
        .args([
            "action", "add", "1", "--description", "do it", "--owner", "alice",
        ])
        .assert()
        .success();

    pirs()
        .current_dir(temp.path())
        .arg("metrics")
        .assert()
        .success()
        .stdout(predicate::str::contains("Incidents: 1"))
        .stdout(predicate::str::contains("Open: 1"))
        .stdout(predicate::str::contains("High"));
}

#[test]
fn metrics_json_output_has_total_field() {
    let temp = assert_fs::TempDir::new().unwrap();
    pirs().current_dir(temp.path()).arg("init").assert().success();
    pirs()
        .current_dir(temp.path())
        .args(["--json", "metrics"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"total\": 0"));
}

// ---------------------------------------------------------------------------
// REQ-RPT-004: pirs doctor --language
// ---------------------------------------------------------------------------

#[test]
fn doctor_language_warns_on_blameful_phrases() {
    let temp = assert_fs::TempDir::new().unwrap();
    pirs().current_dir(temp.path()).arg("init").assert().success();
    pirs()
        .current_dir(temp.path())
        .args([
            "new",
            "Bad outage",
            "--problem",
            "alice was careless and dropped the ball",
            "--no-edit",
        ])
        .assert()
        .success();

    pirs()
        .current_dir(temp.path())
        .args(["doctor", "--language"])
        .assert()
        .success()
        .stdout(predicate::str::contains("blame-oriented phrase"))
        .stdout(predicate::str::contains("careless"));
}

#[test]
fn doctor_without_language_flag_does_not_warn_on_blame_phrases() {
    let temp = assert_fs::TempDir::new().unwrap();
    pirs().current_dir(temp.path()).arg("init").assert().success();
    pirs()
        .current_dir(temp.path())
        .args([
            "new",
            "Outage",
            "--problem",
            "alice was careless",
            "--no-edit",
        ])
        .assert()
        .success();

    pirs()
        .current_dir(temp.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("blame-oriented").not());
}

// ---- REQ-COMP-001..003: shell completions (ADR-0009) ----

#[test]
fn req_comp_001_completions_bash_emits_script_to_stdout() {
    pirs()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_pirs"))
        .stdout(predicate::str::contains("COMPREPLY"));
}

#[test]
fn req_comp_002_completions_out_dir_writes_canonical_filename() {
    let temp = assert_fs::TempDir::new().unwrap();
    let dir = temp.child("comp");
    pirs()
        .args(["completions", "zsh", "--out-dir"])
        .arg(dir.path())
        .assert()
        .success();
    dir.child("_pirs").assert(predicate::path::is_file());
}

#[test]
fn req_comp_003_unknown_shell_is_rejected() {
    pirs()
        .args(["completions", "tcsh"])
        .assert()
        .failure();
}

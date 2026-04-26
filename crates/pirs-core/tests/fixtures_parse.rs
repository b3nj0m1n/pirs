//! Fixture corpus smoke tests (REQ-FIX-001..003 / ADR-0011).
//!
//! Walks every Markdown file under `tests/fixtures/pir-corpus/` and
//! asserts that:
//!
//! * It parses successfully through `pirs_core::parse::Parser`.
//! * Each canonical incident type has at least one fixture.
//! * Exactly one `5 Whys` entry per fixture is tagged `as_root_cause`.
//! * No common real-secret prefixes (`AKIA…`, `ghp_…`, `xox…`) appear
//!   anywhere in the fixture content.

use pirs_core::{IncidentType, Parser};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn corpus_root() -> PathBuf {
    // CARGO_MANIFEST_DIR points at crates/pirs-core/.
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .join("tests")
        .join("fixtures")
        .join("pir-corpus")
}

fn collect_fixtures() -> Vec<PathBuf> {
    WalkDir::new(corpus_root())
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().is_some_and(|x| x == "md"))
        .map(|e| e.into_path())
        .collect()
}

#[test]
fn req_fix_001_every_fixture_parses() {
    let parser = Parser::new();
    let fixtures = collect_fixtures();
    assert!(
        !fixtures.is_empty(),
        "expected fixtures under {}",
        corpus_root().display()
    );
    for path in fixtures {
        let pir = parser
            .parse_file(&path)
            .unwrap_or_else(|e| panic!("failed to parse {}: {e}", path.display()));
        assert!(
            pir.number >= 1,
            "{} should have a non-zero PIR number",
            path.display()
        );
        assert!(
            pir.root_cause.is_some(),
            "{} must declare a root_cause",
            path.display()
        );
        assert!(
            !pir.five_whys.is_empty(),
            "{} must contain a 5-Whys chain",
            path.display()
        );
    }
}

#[test]
fn req_fix_002_corpus_covers_every_incident_type() {
    let parser = Parser::new();
    let mut seen: Vec<IncidentType> = Vec::new();
    for path in collect_fixtures() {
        let pir = parser
            .parse_file(&path)
            .unwrap_or_else(|e| panic!("parse {}: {e}", path.display()));
        if !seen.contains(&pir.incident_type) {
            seen.push(pir.incident_type);
        }
    }
    for required in [
        IncidentType::Development,
        IncidentType::Production,
        IncidentType::Security,
        IncidentType::Process,
    ] {
        assert!(
            seen.contains(&required),
            "fixture corpus is missing a {required:?} PIR; have {seen:?}"
        );
    }
}

#[test]
fn req_fix_003_no_real_secret_prefixes() {
    // Common real-credential prefixes that must never appear in fixtures.
    // Tokens here are deliberately split so this test file itself does
    // not look like a credential to scanners.
    let needles: &[&str] = &[
        "AKIA",            // AWS access key prefix
        "ASIA",            // AWS STS temporary key prefix
        concat!("ghp", "_"),  // GitHub personal access token
        concat!("ghs", "_"),  // GitHub server-to-server token
        concat!("gho", "_"),  // GitHub OAuth token
        concat!("xox", "b-"), // Slack bot token
        concat!("xox", "p-"), // Slack user token
        concat!("-----BEGIN ", "PRIVATE KEY"),
        concat!("-----BEGIN ", "RSA PRIVATE KEY"),
        concat!("-----BEGIN ", "OPENSSH PRIVATE KEY"),
        concat!("-----BEGIN ", "EC PRIVATE KEY"),
    ];
    for path in collect_fixtures() {
        let body = std::fs::read_to_string(&path).unwrap();
        for needle in needles {
            assert!(
                !body.contains(needle),
                "fixture {} contains forbidden secret prefix {needle:?}",
                path.display()
            );
        }
    }
}

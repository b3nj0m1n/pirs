# Review: feat/json-import-redacted-export

Date: 2026-04-26

Branch: `feat/json-import-redacted-export`

Base: `origin/main`

Design ADR: [ADR 0007](../adr/0007-keep-json-pir-import-and-redaction-policy-in-pirs-core.md)

## Scope

This review covers JSON-PIR v1 import and redacted JSON export:

- `pirs export json --redact`
- `pirs import json <FILE>`
- `pirs import json -`
- `--dry-run` and `--overwrite` import behavior
- JSON-PIR parse/version validation and redaction error handling

## Validation

Final validation commands passed:

```sh
target/debug/pirs run --agent "GitHub Copilot" -- cargo test -q
target/debug/pirs run --agent "GitHub Copilot" -- cargo clippy --all-targets --all-features -- -D warnings
target/debug/pirs run --agent "GitHub Copilot" -- cargo fmt --check
```

The final test suite contains 28 tests.

## Code Review Report

Reviewer: fresh-context code review subagent

Verdict: approved for merge.

Critical findings: none.

High findings: none.

Confirmed remediations:

- JSON parse errors are sanitized to line and column only, avoiding unredacted input leakage.
- JSON-PIR schema URL now points at the repository's raw GitHub schema file.
- Invalid redaction-pattern errors report only the pattern index, not the pattern text.

Clean areas:

- Redaction logic remains conditional on `--redact`; normal export is unchanged.
- Import dry-run returns before any repository write.
- Existing PIR numbers skip by default and overwrite only with `--overwrite`.
- Regression tests cover parse-error sanitization and redaction-pattern error sanitization.

Residual non-blocking concern:

- Import input size is not capped. This is accepted for the local CLI MVP and can be addressed in a follow-up.

## Security Review Report

Reviewer: fresh-context security review subagent

Verdict: approved for merge.

Critical findings: none.

High findings: none.

Confirmed remediations:

- Malformed JSON import errors do not echo embedded redactable content such as token-like strings.
- Invalid regex errors do not echo configured redaction pattern text.
- Schema metadata no longer uses the placeholder `example.invalid` URL.

Clean areas:

- `--redact` masks configured regex matches and sensitive fields recursively.
- Import accepts only an explicit path or `-` for stdin; no glob expansion is introduced.
- Dry-run writes nothing.
- Duplicate PIR numbers in import input are rejected.
- Unsupported JSON-PIR versions are rejected.

Residual non-blocking risks:

- Large import inputs can consume memory because file/stdin reads are currently unbounded.
- Timestamp coherence is not rejected at import time; later lint/review-gate checks can catch inconsistent PIRs.
- Empty bulk imports are allowed and report zero planned changes.

## Rollout Plan

1. Merge the feature branch after review approval.
2. Use `pirs export json` as before for normal exports.
3. Use `pirs export json --redact` only after configuring `[privacy].redaction_patterns` or `[privacy].sensitive_fields` in `pirs.toml`.
4. Use `pirs import json <FILE> --dry-run` before importing into an existing repository.
5. Use `--overwrite` only after reviewing the dry-run output for existing-number collisions.
6. Avoid multi-GB or untrusted streaming inputs until import size limits are added.
7. Serialize `--overwrite` imports at the script/process level until repository file locking exists.

## Rollback Plan

Revert the feature branch merge if import or redacted export behavior causes regressions. The change is reversible-easy: it adds CLI flags/subcommands and core helper functions but does not migrate existing PIR files or alter the Markdown storage format. Existing repositories remain readable by the pre-feature code.
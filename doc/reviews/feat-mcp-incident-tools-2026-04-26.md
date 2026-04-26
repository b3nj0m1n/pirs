# Review Gate: feat/mcp-incident-tools

Date: 2026-04-26
Branch: `feat/mcp-incident-tools`
Scope: `main..HEAD`

## Verification Before Review

- `rustfmt --edition 2024 --check crates/pirs-core/src/related.rs crates/pirs-core/tests/related.rs crates/pirs/src/mcp.rs crates/pirs/tests/mcp.rs`: passed
- `cargo clippy --all-targets --all-features -- -D warnings`: passed
- `cargo test --workspace`: passed, 58 tests
- `mdbook build book`: passed

## Code Review Report

### Summary

Adds `get_incident_metrics` and `suggest_related_pirs` MCP tools backed by a new pure `pirs-core::related` module that scores related PIRs deterministically while enforcing the REQ-MCP-003C privacy boundary at the type level. Overall the change is small, well-scoped, ADR-aligned, and adheres to the requirements in spirit and shape.

### Scope Reviewed

- `crates/pirs-core/src/related.rs`
- `crates/pirs-core/src/lib.rs`
- `crates/pirs/src/mcp.rs`
- `crates/pirs-core/tests/related.rs`
- `crates/pirs/tests/mcp.rs`
- `doc/adr/0012-keep-related-pir-suggestions-in-pirs-core.md`

### Spec Compliance

- REQ-MCP-003A: Filtered metrics plus optional `summary_text` gated by `include_text`. Passed.
- REQ-MCP-003B: Required target PIR, target-not-found error, deterministic ordering, default/capped limit, default min score, bounded score. Passed.
- REQ-MCP-003C: Related suggestions carry only metadata plus bounded signals. No body/root-cause/timeline/5-whys/action excerpts are returned. Passed.

### Findings

#### Critical

None.

#### High

H1: `get_incident_metrics` was reported as inheriting a panic risk from `pir_filters` because `IncidentStatus::from_str`, `IncidentSeverity::from_str`, and `IncidentType::from_str` are called with `unwrap()`.

Disposition: False positive after source inspection. These `FromStr` implementations use `type Err = std::convert::Infallible` and map unknown values to `Custom(String)`, preserving custom status/severity/type support. Added regression coverage in `mcp_get_incident_metrics_accepts_custom_filter_values` to make the contract explicit.

#### Medium

M1: `extract_numbers` extracts all digit runs from PIR relationship link URIs, so a non-PIR relationship URL with digits could inflate a candidate's `has_pir_link` signal.

Disposition: Accepted as a scoring-quality follow-up, not blocking for this feature. The output remains bounded and privacy-safe. Track with future relatedness tuning.

M2: Score saturation at 100 can collapse ordering among highly related candidates, falling back to PIR number.

Disposition: Accepted by ADR-0012 as deterministic and bounded. Documented order is score descending then number ascending; future scoring tuning can add more internal resolution while preserving the external 0..100 score.

#### Low / Nit

L1: `include_text` toggles output named `summary_text`.

Disposition: Accepted. The field mirrors existing CLI text output and is already documented.

L2: JSON Schema does not surface runtime defaults for `limit` and `min_score`.

Disposition: Accepted. Tool descriptions and tests cover behavior.

L3/N1/N2: Small style notes around helper shape and derives.

Disposition: Accepted; no behavior impact.

### Positive Feedback

- Deterministic sorting is explicit and covered by shuffled-input tests.
- Privacy boundary is enforced by response types and verified in core and MCP integration tests.
- MCP handlers remain thin adapters over core functionality.
- ADR-0012 matches the existing pirs-core placement pattern from ADR-0008.

### Verdict

Approve with non-blocking follow-ups.

## Security Review Report

### Scope Reviewed

- `doc/adr/0012-keep-related-pir-suggestions-in-pirs-core.md`
- `crates/pirs-core/src/related.rs`
- `crates/pirs-core/src/lib.rs`
- `crates/pirs/src/mcp.rs`
- `crates/pirs-core/tests/related.rs`
- `crates/pirs/tests/mcp.rs`
- Dependency manifests; no new dependencies were added.

### Findings Summary

- Critical: 0
- High: 0
- Medium: 0
- Low: 1 pre-existing pattern reported, resolved as false positive for this code path after source inspection
- Info: 2

### Detailed Findings

FIND-001: `IncidentStatus/Severity/Type::from_str(...).unwrap()` was reported as a possible panic on invalid filter input.

Disposition: False positive for status/severity/type filters. The enum parsers are infallible and return `Custom(String)` for unknown values. Added `mcp_get_incident_metrics_accepts_custom_filter_values` to confirm graceful custom filtering through MCP.

FIND-002: Loose PIR-link number extraction can inflate `has_pir_link` for relationship links containing unrelated numbers.

Disposition: Info-level scoring precision issue. No privacy/auth impact; keep as future relatedness tuning.

FIND-003: O(N) tokenization per call with no soft cap on repository candidate count.

Disposition: Accepted by ADR-0012. Output is capped at 20; future indexing can be considered if repo size grows.

### Explicit No-Findings Areas

- Authentication/authorization: no new auth-relevant code; tools inherit existing MCP transport posture.
- Secrets/credentials: no hardcoded secrets or environment-variable reads added.
- Injection: no SQL, command execution, or filesystem path construction from tool input in the new code paths.
- Deserialization: typed serde/schemars structs only; no untyped JSON passthrough into trusted code.
- Integer overflow: score arithmetic is bounded and clamped.
- Privacy boundary: related-PIR output cannot carry forbidden body fields without changing response types.
- Memory safety: pure safe Rust, no `unsafe`.

### Verdict

Approve for merge from a security perspective.

## Follow-Up Candidates

- Tighten `extract_numbers` to match PIR-shaped relationship URIs only.
- Consider preserving more internal score resolution before emitting the external capped 0..100 score.
- Consider schema-level default metadata if the MCP framework supports it later.

## Rollout And Rollback

Rollout: merge the feature branch after verification. The new tools are additive MCP read tools and do not alter storage format or existing CLI command behavior.

Rollback: revert the feature commits on this branch. Since no migrations or data mutations are introduced, rollback is limited to removing the new MCP tool registrations, related core module/export, tests, and documentation updates.

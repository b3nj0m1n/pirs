# Pre-merge review — feat/reports-metrics-language-audit (2026-04-26)

Branch implements REQ-RPT-001..004: `pirs generate report`, `pirs generate actions`,
`pirs metrics`, `pirs doctor --language`. Design captured in
[ADR-0008](../adr/0008-keep-reports-metrics-and-language-audit-in-pirs-core.md).

Reviews dispatched as parallel fresh-context subagents per SDLC pre-merge gate.
Recommendation: **merge** (one MEDIUM deferred via PIR-0010, one MEDIUM fixed inline).

## Code review (subagent)

Scope: all 12 staged files, ~750 LOC.

| # | Severity | Finding | Triage |
|---|---|---|---|
| 1 | MEDIUM | Markdown table cell escaping only handles `\|` and `\n`; markdown syntax in cells renders literally. | **Accept** — spec calls for Markdown output; literal rendering is expected. No fix. |
| 2 | MEDIUM | `scan_blameful` emits one issue per phrase match — duplicate warnings if the same phrase appears multiple times in one field. | **Fix inline** — dedup per `(field, phrase)` pair. |
| 3 | LOW | Median uses `i128` intermediate (already overflow-safe). | Accept — already correct. |
| 4 | LOW | `writeln!` errors discarded in `render_metrics_text`. | Accept — String::Write is infallible. |
| 5 | NIT | Top-10 tag truncation has no "+N more" indicator. | Accept — acceptable UX. |
| 6 | NIT | Undated actions cluster together in register sort. | Accept — deterministic secondary sort by PIR number is fine. |

Overall: **MERGE**.

## Security review (subagent)

Scope: same diff, OWASP-aligned threat model.

| # | Severity | Finding | Triage |
|---|---|---|---|
| 1 | MEDIUM | `render_pir_report` does not check `pir.confidentiality` before rendering; sensitive PIR text could leak through `generate report`. | **Defer via PIR-0010** — same gap exists in pre-existing `pirs show`; adding enforcement is out of REQ-RPT-001 scope and needs a cross-command policy decision. |
| 2 | LOW | User PIR text written verbatim into Markdown; ANSI escapes or markdown syntax pass through. | Accept — REQ-RPT-001 specifies Markdown output; PIR text is operator-trusted (same trust model as `pirs show`). |
| 3–8 | INFO/CLEAN | Path traversal, command injection, JSON injection, regex DoS, integer overflow, secrets — all clean. | — |

Overall: **MERGE** (with one deferred MEDIUM tracked).

## Actions taken

- Code review #2 fixed inline before commit (deduplicated blameful-phrase warnings).
- Security review #1 deferred via [PIR-0010](../pir/0010-confidentiality-classification-not-enforced-by-render-pir-report.md) (`type: process`, severity `medium`).
- All other findings explicitly accepted as documented above.

## Verification

- `cargo build --all-targets`: clean.
- `cargo clippy --all-targets --all-features -- -D warnings`: clean.
- `cargo test --all`: 37 tests passed (15 existing CLI + 7 new CLI + 4 MCP + 11 unit).

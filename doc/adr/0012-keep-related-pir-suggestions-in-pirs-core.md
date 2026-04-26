---
number: 12
title: Keep related PIR suggestions in pirs-core
status: accepted
date: 2026-04-26
tags:
  - reversible-easy
  - mcp
  - relatedness
---

# Keep related PIR suggestions in pirs-core

## Context and Problem Statement

REQ-MCP-003 requires the MCP server to expose `get_incident_metrics` and
`suggest_related_pirs`. Metrics already live in `pirs-core` per ADR-0008, but
related-PIR suggestion scoring has no existing implementation. The design
question is whether the related-PIR scoring and response shape should live only
inside the MCP adapter, or in `pirs-core` as a pure transformation over parsed
`Pir` values.

## Decision Drivers

* ADR-0008 already keeps deterministic metrics and report transformations in
  `pirs-core` so CLI and MCP surfaces can reuse the same behavior.
* REQ-MCP-003B requires deterministic related-PIR ordering by score and PIR
  number.
* REQ-MCP-003C forbids returning PIR body excerpts, root-cause text, timeline
  text, 5 Whys text, or action descriptions from related-PIR suggestions.
* The scoring algorithm should be easy to test without starting the MCP stdio
  server.
* The implementation must stay small, dependency-free, and reversible.

## Considered Options

* Keep related-PIR scoring local to `crates/pirs/src/mcp.rs`.
* Add a pure relatedness helper and response types to `pirs-core`.
* Split the contract: keep response types and ordering in `pirs-core`, but keep
  scoring in the MCP adapter.

## Decision Outcome

Chosen option: **Add a pure relatedness helper and response types to
`pirs-core`**.

`pirs-core` will expose a small, deterministic helper that accepts a parsed PIR
slice, a target PIR number, and suggestion options. It will return an error when
the target PIR is absent. Otherwise, it returns typed suggestions containing only
metadata allowed by REQ-MCP-003C: number, title, status, severity, incident
type, tags, score, and bounded matching signals. The helper owns input-order
normalization by sorting candidates by PIR number before scoring, so stable
results do not depend on filesystem or caller ordering. The MCP layer will
remain an imperative shell that opens the repository per call, parses tool
arguments, invokes the core helper, and serializes the typed result.

The score is an unsigned integer in the range 0..100 for a given scoring
version. The helper counts shared normalized text tokens, but never returns the
tokens themselves. Shared tag signals are capped to five tag values per
suggestion. The numeric meaning of the score is not a long-term semantic
contract: future versions may tune weights while preserving bounded results,
the descending-score then ascending-number ordering rule, and the privacy
boundary.

### Positive Consequences

* Related-PIR scoring is unit-testable without temp repositories or MCP stdio
  subprocesses.
* The privacy boundary is enforced by the response type: forbidden body fields
  are not carried by the suggestion value.
* The closed signal structure prevents free-form body text from being smuggled
  through explanatory match details.
* The design follows ADR-0008's library-first pattern for deterministic
  transformations over `Pir` values.
* A future CLI or report surface can reuse the same helper without moving logic
  out of the MCP adapter later.

### Negative Consequences

* `pirs-core` gains a small public API before a second consumer exists.
* The scoring heuristic becomes visible to library users, even though the
  numeric score is intentionally evolvable.
* Tokenization and weighting rules become part of core library behavior and will
  need regression tests when tuned.
* Per-call `Repository::open` plus full-repository scoring is O(N) in PIR count;
  repositories with very large PIR histories may need a future indexed design.

### Confirmation

Compliance is confirmed by:

* `pirs-core` unit tests for scoring, deterministic ordering, limit capping,
  minimum score filtering, target exclusion, and privacy-safe serialization.
* `pirs-core` unit tests that shuffled input produces identical output and that
  the serialized suggestion shape contains only fields allowed by REQ-MCP-003C.
* MCP integration tests proving `tools/list` advertises both tools and
  `tools/call` returns the metrics and related-PIR response shapes defined in
  REQ-MCP-003A..003C.
* Review of `crates/pirs/src/mcp.rs` to verify the MCP handlers remain thin
  adapters over `Repository::list`, `compute_metrics`, and the relatedness
  helper.

## Pros and Cons of the Options

### Keep related-PIR scoring local to `crates/pirs/src/mcp.rs`

* Good, because it minimizes public API growth in `pirs-core`.
* Good, because a volatile heuristic can change without affecting library
  consumers.
* Bad, because scoring would be tested mostly through slower MCP integration
  tests.
* Bad, because the privacy response shape would depend on adapter discipline
  instead of a typed core value.
* Bad, because a future CLI related-PIR command would need to move or duplicate
  the logic.

### Add a pure relatedness helper and response types to `pirs-core`

* Good, because deterministic scoring over parsed `Pir` values is domain logic,
  not transport logic.
* Good, because typed suggestions can omit forbidden fields by construction.
* Good, because direct unit tests can cover ranking and edge cases cheaply.
* Bad, because it expands the public library surface before a second consumer is
  present.
* Bad, because score tuning requires care to preserve documented guarantees.

### Split contract and scoring across core and MCP

* Good, because it preserves a type-level response boundary while keeping the
  volatile score weights adapter-local.
* Good, because it reduces public API commitment compared with a full helper.
* Bad, because scoring remains harder to unit-test.
* Bad, because the architecture still leaves deterministic content-derived
  behavior in the transport adapter.

## More Information

Related decisions: ADR-0004 (per-call repository open in MCP handlers),
ADR-0005 (single MCP module), ADR-0008 (reports and metrics in `pirs-core`).

Revisit this decision if related-PIR suggestions require persistent indexes,
embeddings, external search services, or access-controlled redaction policies.
---
number: 8
title: Keep reports, metrics, and blameless language audit in pirs-core
status: accepted
date: 2026-04-26
tags:
  - reversible-easy
  - reports
  - metrics
  - lint
---

# Keep reports, metrics, and blameless language audit in pirs-core

## Context and Problem Statement

REQ-RPT-001..004 require four outputs: a per-PIR Markdown report
(`generate report <PIR>`), a cross-PIR action register (`generate actions`),
a repository-wide incident metrics summary (`pirs metrics`), and a blame-
oriented language warning surface (`pirs doctor --language`). Each is a pure
transformation over already-parsed `Pir` values: the CLI only needs to load
the repository, call into the library, and print or stream the result.

The design question mirrors ADR-0007: do reports and metrics live in command
modules, or as library functions in `pirs-core`? Keeping rendering and
aggregation in the library lets MCP tools (REQ-MCP-007 `get_incident_metrics`)
reuse the same code, and lets unit tests cover formatting and statistics
without filesystem setup.

## Decision Drivers

* Mirror ADR-0007: CLI is the imperative shell; transformations live in
  `pirs-core`.
* `get_incident_metrics` is on the MCP roadmap and will need the same
  metrics computation — write it once.
* Reports, action registers, and metrics are deterministic over a list of
  `Pir` values; they belong in pure functions that are trivial to unit-test.
* The blameless language audit is another lint pass, not a new subsystem;
  extending `lint.rs` keeps doctor's wiring uniform.
* Keep the change small and reversible: no new dependency, no new format
  version, no schema change.

## Considered Options

* **A. Inline rendering inside command modules.** Quick to write, but
  duplicates work when MCP tools land and forces test setup through the
  filesystem.
* **B. Add `report` and `metrics` modules to `pirs-core`; extend `lint.rs`
  with language patterns.** Library renders strings and structs; CLI
  prints them; future MCP tools call the same functions.

## Decision Outcome

Chosen option: **B**. Adds two small modules to `pirs-core`
(`report.rs`, `metrics.rs`) and a `language` lint pass alongside existing
rules. Three new commands (`generate report`, `generate actions`, `metrics`)
and a new flag (`doctor --language`) wire them up.

### Positive Consequences

* Reuse for MCP `get_incident_metrics` is free.
* Pure functions are unit-testable without `assert_fs`.
* Doctor's existing reporting loop covers language warnings unchanged.

### Negative Consequences

* Two new files in `pirs-core`; small surface-area growth.
* Markdown report rendering ties the library to a specific output template
  shape — acceptable because the template is documented in the spec.

## Links

* Supersedes / amends: none
* Related: ADR-0007 (same principle for JSON-PIR policy)
* Requirements: REQ-RPT-001, REQ-RPT-002, REQ-RPT-003, REQ-RPT-004

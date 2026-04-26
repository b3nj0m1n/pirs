---
number: 1
title: Schemars 0.8 vs 1.x mismatch with tower-mcp 0.10
status: Reviewed
severity: Low
incident_type: Development
problem_statement: 'Initial mcp.rs build produced 26 ExtractorHandler trait-bound errors that masked a schemars major-version mismatch: Cargo.toml pinned schemars 0.8 while tower-mcp 0.10 requires schemars 1.x. JsonSchema impls did not satisfy the FromToolRequest+HasSchema bounds.'
detected_at: 2026-04-26T05:09:54.575649Z
resolved_at: 2026-04-26T05:10:27.645657Z
time_to_resolve: PT33S
people_involved:
- name: GitHub Copilot
  type: agent
timeline:
- at: 2026-04-26T05:09:54.575649Z
  actor: GitHub Copilot
  type: detected
  description: incident detected
- at: 2026-04-26T05:10:27.520907Z
  actor: GitHub Copilot
  type: investigated
  description: Fetched ExtractorHandler trait docs
- at: 2026-04-26T05:10:27.538517Z
  actor: GitHub Copilot
  type: investigated
  description: Changed helper return type to tower_mcp::Result; same 26 errors
- at: 2026-04-26T05:10:27.555365Z
  actor: GitHub Copilot
  type: investigated
  description: Identified schemars 0.8 vs 1.x version skew via cargo tree
- at: 2026-04-26T05:10:27.572186Z
  actor: GitHub Copilot
  type: fix-applied
  description: Bumped workspace schemars to 1; build clean
- at: 2026-04-26T05:10:27.645657Z
  actor: pirs
  type: resolved
  description: status -> Resolved
five_whys:
- question: Why did 13 tool registrations fail with the same trait bound?
  answer: All 13 closures called .extractor_handler with Json<T> args; T's JsonSchema impl came from schemars 0.8 but tower-mcp's HasSchema bound expected schemars 1.x
- question: Why was the wrong schemars version chosen?
  answer: Picked 0.8 from memory without checking tower-mcp 0.10's transitive dependency tree
actions:
- id: ACT-001
  description: When adding a crate that uses schemars/serde-derive ecosystem, run cargo tree before writing handlers
  owner: GitHub Copilot
  owner_type: agent
  due: 2026-12-31
  status: Open
impact: 'Build break of `cargo build -p pirs` with 26 errors after registering 13 MCP tool handlers; ~15 minutes of investigation. Caught at compile time on feature branch — no production impact.'
root_cause: Picked 0.8 from memory without checking tower-mcp 0.10's transitive dependency tree
confidentiality: Internal
---

# 1. Schemars 0.8 vs 1.x mismatch with tower-mcp 0.10

> Type: Development · Severity: Low

## Problem Statement

Initial mcp.rs build produced 26 ExtractorHandler trait-bound errors that masked a schemars major-version mismatch: Cargo.toml pinned schemars 0.8 while tower-mcp 0.10 requires schemars 1.x. JsonSchema impls did not satisfy the FromToolRequest+HasSchema bounds.

## Impact

* Build break: `cargo build -p pirs` failed with 26 errors at the
  point all 13 MCP tool handlers were registered. Nothing else in the
  workspace was affected.
* Wasted investigation time: roughly 15 minutes spent inspecting
  `ExtractorHandler` trait bounds and helper return types before the
  real cause (a transitive major-version skew) was identified.
* No production impact — caught at compile time on a feature branch
  before any commit was pushed.

## People and Systems Involved

* GitHub Copilot (agent) — author of the failing change and the fix.
* Crates: `tower-mcp` 0.10, `schemars` 0.8 (initial pin) → 1.x (fix),
  `serde`, `serde_json`.

## Timeline

_Populated via `pirs timeline add`._

* `2026-04-26T05:09:54Z` — detected (build failure: 26 errors).
* Investigated `ExtractorHandler` trait docs.
* Investigated by changing helper return type to
  `tower_mcp::Result<CallToolResult>` — same 26 errors persisted.
* Investigated via `cargo tree` and identified the schemars 0.8 vs
  1.x version skew.
* Fix applied: bumped workspace `schemars` to `"1"`; build clean.
* `2026-04-26T05:10:27Z` — resolved.

## Detection and Resolution Timing

* MTTR: **PT33S** (33 seconds wall clock from `detected` to
  `resolved`). Investigation effort was longer than wall-clock time
  suggests, because timeline events were backfilled after the fix.

## 5 Whys

1. **Why did 13 tool registrations fail with the same trait bound?**
   All 13 closures called `.extractor_handler` with `Json<T>` args;
   `T`'s `JsonSchema` impl came from `schemars` 0.8 but tower-mcp's
   `HasSchema` bound expected `schemars` 1.x.
2. **Why was the wrong schemars version chosen?** *(root cause)*
   Picked `0.8` from memory without checking tower-mcp 0.10's
   transitive dependency tree.

## Actions

* **ACT-001** — When adding a crate that uses the schemars/serde-derive
  ecosystem, run `cargo tree` before writing handlers. *(Owner:
  GitHub Copilot, due 2026-12-31, Open.)*

## Lessons Learned

* The error message (`ExtractorHandler` trait bound not satisfied)
  pointed at a symptom, not the cause. When 13 nearly-identical
  closures fail with the same bound, suspect a shared transitive
  dependency, not the closure shapes.
* `cargo tree -p <new-crate> -e features` should be a reflex when
  pulling in a new derive-heavy dependency, before writing any code
  against its API.
* Major-version skews on schema/serde traits are silent at the
  `Cargo.toml` level — both versions resolve, both compile in
  isolation, but trait impls from one don't satisfy bounds from the
  other.

## Links

* Pull request: https://github.com/b3nj0m1n/pirs/pull/1
* Fix commit: `af5b170` (feat(mcp): add MCP server with read + write tools)
* Related: PIR-0002 (also opened during the same MCP-server bootstrap;
  different root cause but the same overall pattern of insufficient
  pre-commit verification).











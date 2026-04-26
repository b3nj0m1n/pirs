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
impact: _What systems, tests, environments, or workflows were affected?_
root_cause: Picked 0.8 from memory without checking tower-mcp 0.10's transitive dependency tree
confidentiality: Internal
---

# 1. Schemars 0.8 vs 1.x mismatch with tower-mcp 0.10

> Type: Development · Severity: Low

## Problem Statement

Initial mcp.rs build produced 26 ExtractorHandler trait-bound errors that masked a schemars major-version mismatch: Cargo.toml pinned schemars 0.8 while tower-mcp 0.10 requires schemars 1.x. JsonSchema impls did not satisfy the FromToolRequest+HasSchema bounds.

## Impact

_What systems, tests, environments, or workflows were affected?_

## People and Systems Involved

_Humans, agents, teams, or systems involved (blameless)._

## Timeline

_Ordered events, populated via `pirs timeline add`._

## Detection and Resolution Timing

_Time to discover and time to resolve, derived from timestamps._

## 5 Whys

_Add ordered entries via `pirs why add`._

## Actions

_Add follow-up actions via `pirs action add`._

## Lessons Learned

_What went well, what went wrong, where we got lucky._

## Links

_Typed evidence links: commits, PRs, issues, dashboards, runbooks._











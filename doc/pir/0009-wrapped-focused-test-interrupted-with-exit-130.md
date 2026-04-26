---
number: 9
title: wrapped focused test interrupted with exit 130
status: Resolved
severity: Low
incident_type: Development
problem_statement: A pirs run wrapper around the focused invalid-pattern regression test was interrupted with exit code 130 before diagnostics were produced. The same focused cargo test was then run directly and passed. This indicates wrapper or terminal instability, not a code regression.
detected_at: 2026-04-26T06:10:15.36934Z
resolved_at: 2026-04-26T06:10:42.737806Z
time_to_resolve: PT27S
people_involved:
- name: GitHub Copilot
  type: agent
timeline:
- at: 2026-04-26T06:10:15.36934Z
  actor: GitHub Copilot
  type: detected
  description: incident detected
- at: 2026-04-26T06:10:42.718123Z
  actor: GitHub Copilot
  type: note
  description: The focused regression test passed directly, and the full wrapped test suite subsequently passed with 28 tests. The interruption was isolated to that wrapper invocation.
- at: 2026-04-26T06:10:42.737806Z
  actor: pirs
  type: resolved
  description: status -> Resolved
impact: _What systems, tests, environments, or workflows were affected?_
tags:
- agent-audit
- test-wrapper
confidentiality: Internal
---

# 9. wrapped focused test interrupted with exit 130

> Type: Development · Severity: Low

## Problem Statement

A pirs run wrapper around the focused invalid-pattern regression test was interrupted with exit code 130 before diagnostics were produced. The same focused cargo test was then run directly and passed. This indicates wrapper or terminal instability, not a code regression.

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



---
number: 5
title: zsh status variable broke wrapped test exit marker
status: Resolved
severity: Low
incident_type: Process
problem_statement: After the CLI tests passed under pirs run, the shell wrapper attempted to assign to zsh's read-only status variable while echoing the exit marker. That made the overall terminal command exit 1 even though the test command succeeded. Recovery is to use a different variable name such as exit_code.
detected_at: 2026-04-26T05:57:17.005443Z
resolved_at: 2026-04-26T05:59:57.326782Z
time_to_resolve: PT2M40S
people_involved:
- name: GitHub Copilot
  type: agent
timeline:
- at: 2026-04-26T05:57:17.005443Z
  actor: GitHub Copilot
  type: detected
  description: incident detected
- at: 2026-04-26T05:59:26.454835Z
  actor: GitHub Copilot
  type: note
  description: The initial pirs new command for this audit record included backticks around a shell variable name, and zsh attempted command substitution before pirs received the text. The record was inspected and the recovery is to avoid backticks in shell-quoted CLI arguments or use single quotes.
- at: 2026-04-26T05:59:57.326782Z
  actor: pirs
  type: resolved
  description: status -> Resolved
impact: _Workflows, teams, deliverables, customer commitments affected._
tags:
- agent-audit
- test-wrapper
confidentiality: Internal
---

# 5. zsh status variable broke wrapped test exit marker

> Type: Process · Severity: Low

## Problem Statement

After the CLI tests passed under pirs run, the shell wrapper attempted to assign to zsh's read-only status variable while echoing the exit marker. That made the overall terminal command exit 1 even though the test command succeeded. Recovery is to use a different variable name such as exit_code.

## Impact

_Workflows, teams, deliverables, customer commitments affected._

## People and Systems Involved

## Timeline

## Detection and Resolution Timing

## 5 Whys

## Actions

## Lessons Learned

## Links



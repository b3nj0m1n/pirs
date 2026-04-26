---
number: 3
title: macOS grep rejected ADR/PIR placeholder directory arguments
status: Resolved
severity: Low
incident_type: Process
problem_statement: The SDLC placeholder-check command was run as plain grep against doc/adr/ and doc/pir/ directories on macOS, which exits with code 2 because recursive search was not requested. The recovery is to use rg or recursive grep for the same placeholder pattern before staging ADR/PIR artifacts.
detected_at: 2026-04-26T05:52:13.09872Z
resolved_at: 2026-04-26T05:59:57.1263Z
time_to_resolve: PT7M44S
people_involved:
- name: GitHub Copilot
  type: agent
timeline:
- at: 2026-04-26T05:52:13.09872Z
  actor: GitHub Copilot
  type: detected
  description: incident detected
- at: 2026-04-26T05:53:00.174198Z
  actor: GitHub Copilot
  type: note
  description: ripgrep was not installed, and the first timeline retry used a --type flag unsupported by the current built binary. Recovered by using the binary's default timeline event type and grep -R for recursive placeholder scanning.
- at: 2026-04-26T05:59:57.1263Z
  actor: pirs
  type: resolved
  description: status -> Resolved
impact: _Workflows, teams, deliverables, customer commitments affected._
tags:
- agent-audit
- placeholder-check
confidentiality: Internal
---

# 3. macOS grep rejected ADR/PIR placeholder directory arguments

> Type: Process · Severity: Low

## Problem Statement

The SDLC placeholder-check command was run as plain grep against doc/adr/ and doc/pir/ directories on macOS, which exits with code 2 because recursive search was not requested. The recovery is to use rg or recursive grep for the same placeholder pattern before staging ADR/PIR artifacts.

## Impact

_Workflows, teams, deliverables, customer commitments affected._

## People and Systems Involved

## Timeline

## Detection and Resolution Timing

## 5 Whys

## Actions

## Lessons Learned

## Links



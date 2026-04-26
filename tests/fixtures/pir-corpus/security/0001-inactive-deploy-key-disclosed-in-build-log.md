---
number: 1
title: 'Inactive deploy key disclosed in build log'
status: Resolved
severity: Medium
incident_type: Security
problem_statement: |
  A read-only deploy key for the `web-static` repository was printed to
  a public GitHub Actions build log when an unrelated debug step echoed
  the environment. The key had been rotated out of active use 11 days
  earlier and was still listed on the repository as "deactivated".
occurred_at: 2026-04-03T11:08:00Z
detected_at: 2026-04-03T11:25:42Z
resolved_at: 2026-04-03T11:58:00Z
detection_method: Secret-scanning alert
people_involved:
  - name: Security on-call
    type: human
    role: incident commander
  - name: Platform team
    type: team
    role: rotation
timeline:
  - at: 2026-04-03T11:08:00Z
    actor: GitHub Actions
    type: detected
    description: Build step `print-env` echoed the deploy key to a public log.
  - at: 2026-04-03T11:25:42Z
    actor: secret-scanner
    type: detected
    description: Synthetic key prefix `EXAMPLE_TOKEN_AAAA...` matched.
  - at: 2026-04-03T11:32:00Z
    actor: Security on-call
    type: investigated
    description: Confirmed key was already deactivated; scope was limited.
  - at: 2026-04-03T11:48:00Z
    actor: Platform team
    type: action_added
    description: Removed the deactivated key from the repository entirely.
  - at: 2026-04-03T11:58:00Z
    actor: pirs
    type: resolved
    description: Log redacted; key fully removed; status -> Resolved.
five_whys:
  - question: Why was the key in the log?
    answer: A debug step echoed the entire environment.
  - question: Why did the debug step echo the entire environment?
    answer: It was added during an unrelated build-flake investigation and not removed.
  - question: Why was it not removed?
    answer: The branch with the debug step was merged without a final cleanup pass.
  - question: Why was the merge allowed?
    answer: There was no CI lint preventing `printenv`-style steps in workflows.
root_cause: There was no CI lint preventing `printenv`-style steps in workflows.
actions:
  - id: A1
    description: Add a workflow lint forbidding bulk env dumps in build steps.
    owner: Platform team
    owner_type: team
    status: Open
    due: 2026-05-10
  - id: A2
    description: Audit and remove all deactivated deploy keys repository-wide.
    owner: Security on-call
    owner_type: team
    status: Open
    due: 2026-04-30
links:
  - kind: Issue
    uri: https://example.invalid/security/issues/77
    description: Tracking issue (private).
  - kind: Runbook
    uri: https://example.invalid/runbooks/secret-disclosure
    description: Secret-disclosure response runbook.
tags:
  - secrets
  - ci
  - blameless
confidentiality: Confidential
---

# 1. Inactive deploy key disclosed in build log

> Type: security · Severity: medium

## Problem Statement

A read-only deploy key for the `web-static` repository was printed to a
public GitHub Actions build log when an unrelated debug step echoed the
environment. The key had been rotated out of active use 11 days earlier
and was still listed on the repository as "deactivated".

## Impact

No active credential exposed. Reputation / process risk only. The
disclosed value is the synthetic placeholder `EXAMPLE_TOKEN_AAAA...`
recorded in this fixture; the actual rotated key is not reproduced.

## People and Systems Involved

Security on-call, Platform team.

## Timeline

See frontmatter; populated via `pirs timeline add`.

## Detection and Resolution Timing

Detected within 17 minutes 42 seconds; resolved 50 minutes after onset.

## 5 Whys

See frontmatter `five_whys`.

## Actions

See frontmatter `actions`.

## Lessons Learned

Deactivated does not mean removed. Workflow steps with broad
environment access need lint coverage to catch debug-leftover patterns.

## Links

See frontmatter `links`.

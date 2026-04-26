---
number: 1
title: 'Release runbook step skipped during cutover'
status: Resolved
severity: Low
incident_type: Process
problem_statement: |
  During the v1.7 release cutover, the runbook step "freeze schema
  migrations on read replicas" was skipped because the operator was
  paged into a different incident mid-rollout. The release succeeded,
  but the gap was discovered only during the post-release review.
occurred_at: 2026-04-18T15:00:00Z
detected_at: 2026-04-18T16:30:00Z
resolved_at: 2026-04-18T17:10:00Z
detection_method: Post-release review
people_involved:
  - name: Release manager
    type: human
    role: rollout owner
  - name: Database team
    type: team
    role: schema reviewer
timeline:
  - at: 2026-04-18T15:00:00Z
    actor: Release manager
    type: investigated
    description: Started v1.7 release per runbook.
  - at: 2026-04-18T15:18:00Z
    actor: Release manager
    type: note
    description: Paged into a parallel incident; resumed rollout 28 min later without re-reading runbook.
  - at: 2026-04-18T16:30:00Z
    actor: Release manager
    type: detected
    description: Post-release review found schema-migration freeze was never applied.
  - at: 2026-04-18T17:10:00Z
    actor: pirs
    type: resolved
    description: No data corruption; freeze applied retroactively. status -> Resolved.
five_whys:
  - question: Why was the freeze step skipped?
    answer: The operator was paged into another incident mid-rollout.
  - question: Why did resuming skip the step?
    answer: The runbook had no resumption checkpoint; the operator continued from memory.
  - question: Why did the runbook have no resumption checkpoint?
    answer: Runbooks were written assuming uninterrupted execution.
root_cause: Runbooks were written assuming uninterrupted execution.
actions:
  - id: A1
    description: Add explicit resumption checkpoints to all release runbooks.
    owner: Release engineering
    owner_type: team
    status: Open
    due: 2026-05-30
  - id: A2
    description: Define an interrupt protocol — pause runbook before joining another incident.
    owner: Release engineering
    owner_type: team
    status: Open
    due: 2026-05-30
links:
  - kind: Runbook
    uri: https://example.invalid/runbooks/release-v1.7
    description: v1.7 release runbook.
  - kind: PullRequest
    uri: https://example.invalid/repo/pull/118
    description: v1.7 release PR.
tags:
  - process
  - runbook
  - blameless
confidentiality: Internal
---

# 1. Release runbook step skipped during cutover

> Type: process · Severity: low

## Problem Statement

During the v1.7 release cutover, the runbook step "freeze schema
migrations on read replicas" was skipped because the operator was
paged into a different incident mid-rollout. The release succeeded,
but the gap was discovered only during the post-release review.

## Impact

No customer impact. Process gap created an undetected window where a
schema migration could have shipped without a freeze; none did.

## People and Systems Involved

Release manager, Database team.

## Timeline

See frontmatter; populated via `pirs timeline add`.

## Detection and Resolution Timing

Detected at the post-release review (90 minutes after incident start);
resolved 40 minutes later by retroactive freeze.

## 5 Whys

See frontmatter `five_whys`.

## Actions

See frontmatter `actions`.

## Lessons Learned

Runbooks must assume interruption. A resumption checkpoint and an
interrupt protocol prevent silent skips when an operator is paged.

## Links

See frontmatter `links`.

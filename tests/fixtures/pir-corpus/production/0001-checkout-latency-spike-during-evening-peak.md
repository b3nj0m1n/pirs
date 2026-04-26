---
number: 2
title: 'Checkout latency spike during evening peak'
status: Reviewed
severity: High
incident_type: Production
problem_statement: |
  Between 19:42 and 20:14 UTC the `checkout` API p99 latency rose from
  220 ms to 4.8 s, breaching the 1 s SLO. Approximately 1.3% of orders
  saw checkout retries; no orders were lost.
occurred_at: 2026-03-14T19:42:00Z
detected_at: 2026-03-14T19:45:18Z
resolved_at: 2026-03-14T20:14:00Z
detection_method: Synthetic monitor + on-call page
people_involved:
  - name: On-call SRE
    kind: human
    role: incident commander
  - name: Payments team
    kind: team
    role: subject matter expert
  - name: GitHub Copilot
    kind: agent
    role: log triage
timeline:
  - at: 2026-03-14T19:42:00Z
    actor: synthetic-monitor
    type: detected
    description: p99 latency alert fired against `/checkout` endpoint.
  - at: 2026-03-14T19:48:00Z
    actor: On-call SRE
    type: investigated
    description: Identified payment-provider connection-pool saturation.
  - at: 2026-03-14T20:02:00Z
    actor: Payments team
    type: action_added
    description: Raised connection-pool ceiling from 32 to 96.
  - at: 2026-03-14T20:14:00Z
    actor: pirs
    type: resolved
    description: Latency back to baseline; status -> Resolved.
five_whys:
  - question: Why did p99 latency spike?
    answer: Payment-provider client requests queued for available connections.
  - question: Why did requests queue?
    answer: The connection pool was capped at 32, smaller than peak demand.
  - question: Why was the pool capped at 32?
    answer: The default from a 2024 rollout was never revisited.
  - question: Why was it never revisited?
    answer: There was no capacity-review cadence for third-party client pools.
root_cause: There was no capacity-review cadence for third-party client pools.
actions:
  - id: A1
    description: Add quarterly capacity review for all third-party client pools.
    owner: Payments team
    owner_type: team
    status: Open
    due: 2026-06-30
  - id: A2
    description: Emit a connection-pool-saturation metric and SLO.
    owner: SRE
    owner_type: team
    status: Open
    due: 2026-05-30
links:
  - kind: Dashboard
    uri: https://example.invalid/grafana/checkout
    description: Checkout latency dashboard.
  - kind: Runbook
    uri: https://example.invalid/runbooks/checkout-latency
    description: Checkout latency runbook.
tags:
  - latency
  - capacity
  - payments
confidentiality: Internal
---

# 2. Checkout latency spike during evening peak

> Type: production · Severity: high

## Problem Statement

Between 19:42 and 20:14 UTC the `checkout` API p99 latency rose from
220 ms to 4.8 s, breaching the 1 s SLO. Approximately 1.3% of orders
saw checkout retries; no orders were lost.

## Impact

User-visible: ~1.3% of checkout attempts retried during the window. No
order loss; no payment double-charges.

## People and Systems Involved

On-call SRE, Payments team, GitHub Copilot (log triage agent).

## Timeline

See frontmatter; populated via `pirs timeline add`.

## Detection and Resolution Timing

Detected within 3 minutes 18 seconds; resolved 32 minutes after onset.

## 5 Whys

See frontmatter `five_whys`.

## Actions

See frontmatter `actions`.

## Lessons Learned

Default values from old rollouts age silently; we need an explicit
capacity-review cadence for third-party client pools.

## Links

See frontmatter `links`.

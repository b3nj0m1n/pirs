# Severity and Status Taxonomy

This chapter implements **REQ-DOC-008**: a normative reference for the
controlled vocabularies in PIR frontmatter.

## Severity

`severity` records the *impact* of the incident. Pick the highest level that
applies; downgrade only with rationale recorded in the timeline.

| Severity | Meaning | Examples |
|---|---|---|
| `Low` | Local, reversible, no user impact. | Failing developer test on a feature branch; a flaky lint check. |
| `Medium` | Non-trivial scope; user-visible only in narrow conditions. | Failing CI on `main` for a few hours; a logging regression. |
| `High` | Material user impact or service degradation. | Elevated checkout latency; an auth cookie not refreshing for a subset of sessions. |
| `Critical` | Outage, data loss, or active exploitation. | Full service outage; secret leak with confirmed external access; data loss without backup. |

Severity is independent of `incident_type`. A `Process` incident can be
`Critical` (e.g. a release runbook step that, when skipped, took the
production database offline).

## Status lifecycle

`status` records where the PIR is in its lifecycle. The CLI uses these
values for filtering, metrics, and the `Reviewed` review-gate check; it
does **not** enforce a strict predecessor finite state machine, so any
sequence is technically allowed. Treat the order below as the conventional
flow.

```
Open ─► Investigating ─► Mitigated ─► Resolved ─► Reviewed
                                  │
                                  └──── (skip Mitigated only when fix is the mitigation)

Cancelled is a terminal state for incidents that turn out to be non-issues.
```

| Status | Definition | Conventionally requires |
|---|---|---|
| `Open` | Incident is logged but unconfirmed. | `problem_statement` set. |
| `Investigating` | Active diagnosis. | At least one timeline event. |
| `Mitigated` | Symptoms reduced to acceptable level; root cause may still be open. | A mitigation timeline event. |
| `Resolved` | Underlying issue addressed; service nominal. | `resolved_at` set. |
| `Reviewed` | Postmortem complete; lessons recorded. | `problem_statement`, ≥1 timeline event, ≥1 5-Whys entry, ≥1 action item, `resolved_at` (enforced by `pirs doctor --review-gate`). |
| `Cancelled` | Incident determined not to be real. | `--reason` supplied to `pirs status`. |

`pirs doctor --review-gate <N>` reports exactly which prerequisites are
missing for the `Reviewed` transition. Recording a `root_cause` is strongly
recommended for `Reviewed` PIRs but is not currently enforced by the gate.

## Incident type

`incident_type` selects the default report template and clusters metrics:

| Type | Use for | Default template |
|---|---|---|
| `Development` | Failing tests, broken builds, local regressions, dependency-bump fallout. | `templates/development.md` |
| `Production` | User-facing service incidents and outages. | `templates/production.md` |
| `Security` | Confidentiality, integrity, or access-control events. Defaults to `Confidential`. | `templates/security.md` |
| `Process` | Runbook, release, or workflow failures with no code defect. | `templates/process.md` |

A minimal template (`templates/minimal.md`) is available via `pirs new
--template minimal` for quick captures that will be expanded later.

## Confidentiality

`confidentiality` controls export and rendering behaviour:

| Level | Behaviour |
|---|---|
| `Public` | No redaction; safe for issue trackers. |
| `Internal` | Default; assumes inside the project's normal access boundary. |
| `Confidential` | `pirs export json --redact` is the only sanctioned export form. |
| `Restricted` | Treat the on-disk file itself as sensitive; rotate access controls accordingly. |

Security-typed PIRs are auto-classified `Confidential` when created via
`pirs new --type security`.

## Timeline event types

`TimelineEvent.type` accepts the following controlled values plus arbitrary
free-form strings:

`detected`, `investigated`, `mitigated`, `resolved`, `decision`, `note`,
`hypothesis`, `experiment`, `rollback`, `escalated`, `communication`.

Choose the closest match; a free-form string is accepted but loses
filterability in `pirs metrics`.

## Action item status

`ActionItem.status` follows a small lifecycle:

`open` → `in-progress` → `done`, with an out-of-band `cancelled` terminator.
The `pirs actions` register lists open and in-progress actions across all
PIRs; `done` entries appear with the linked evidence.

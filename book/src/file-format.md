# PIR File Format

This chapter implements **REQ-DOC-006**: a reference for the on-disk PIR
format. It is the source of truth for both human authors and LLM agents
generating PIRs through MCP tools.

## File layout

A PIR is a single Markdown file with YAML frontmatter:

```markdown
---
number: 1
title: 'Short imperative incident title'
status: Resolved
severity: Medium
incident_type: Development
problem_statement: |
  Multi-line problem statement.
occurred_at: 2026-04-20T09:12:00Z
detected_at: 2026-04-20T09:14:30Z
resolved_at: 2026-04-20T10:01:00Z
root_cause: >-
  One-sentence root cause once known.
timeline: [...]
five_whys: [...]
actions: [...]
links: [...]
tags: []
confidentiality: Internal
---

# 1. Short imperative incident title

> Type: development · Severity: medium

## Problem Statement

...
```

Files live under the configured PIR directory, defaulting to `doc/pir/`.
Filenames follow `NNNN-slug.md` where `NNNN` is the zero-padded number and
`slug` is generated from the title.

## Frontmatter fields

### Identity

| Field | Type | Notes |
|---|---|---|
| `number` | `u32` | Sequential. Assigned by `pirs new`; do not hand-pick. |
| `title` | string | Short, imperative, no trailing period. |
| `confidentiality` | enum | `Public`, `Internal`, `Confidential`, `Restricted`. |
| `tags` | list of strings | Free-form labels for grouping and search. |

### Classification

| Field | Type | Allowed values |
|---|---|---|
| `status` | enum | See [Severity and Status Taxonomy](./severity-and-status.md) |
| `severity` | enum | `Low`, `Medium`, `High`, `Critical` |
| `incident_type` | enum | `Development`, `Production`, `Security`, `Process` |

### Times

All timestamps are RFC 3339 / ISO 8601 with timezone:

| Field | Meaning |
|---|---|
| `occurred_at` | Best-known start time of the incident. |
| `detected_at` | When the incident was first observed. |
| `resolved_at` | When service or workflow returned to acceptable state. |
| `time_to_discover` | ISO-8601 duration string, derived from `occurred_at`/`detected_at`. |
| `time_to_resolve` | ISO-8601 duration string, derived from `detected_at`/`resolved_at`. |
| `total_duration` | ISO-8601 duration string, derived from `occurred_at`/`resolved_at`. |

The three duration fields are **derived**: `pirs` recomputes them on every
parse, so authors should not edit them manually.

### Narrative

| Field | Type | Notes |
|---|---|---|
| `problem_statement` | string | Required for `Reviewed` status. |
| `impact` | string | Optional summary of who or what was affected. |
| `summary` | string | Optional executive summary written after review. |
| `root_cause` | string | Required for `Reviewed`; set by `pirs why add … --as-root-cause`. |
| `contributing_factors` | list of strings | Non-root factors. |
| `what_went_well` | list of strings | Behaviours to preserve. |
| `what_went_wrong` | list of strings | Process or technical gaps. |
| `where_we_got_lucky` | list of strings | Risks that did not materialise. |
| `detection_method` | string | How the incident was discovered. |

### Structured collections

| Field | Element type |
|---|---|
| `people_involved` | `Actor { name, type, role }` where `type` ∈ `human`, `agent`, `team`, `system`. |
| `timeline` | `TimelineEvent { at, actor, type, description }` ordered by `at`. |
| `five_whys` | `WhyEntry { question, answer }` ordered by appearance. |
| `actions` | `ActionItem { id, description, owner, owner_type, due, status, evidence, notes }`. |
| `links` | `EvidenceLink { kind, uri, description }`. |

`TimelineEvent.type` and `ActionItem.status` are controlled vocabularies; see
the next chapter. `LinkKind` accepts `Commit`, `PullRequest`, `Issue`, `Log`,
`Dashboard`, `Runbook`, `Deployment`, `TestRun`, `ADR`, `PIR`, plus custom
strings.

## Markdown body

Below the frontmatter, the body uses `## Heading 2` sections. The parser
honours these specific headings and pulls their text into the corresponding
frontmatter field if the field is empty:

| Heading | Backed-by field |
|---|---|
| `## Problem Statement` | `problem_statement` |
| `## Impact` | `impact` |
| `## Summary` | `summary` |

Other sections (`## Timeline`, `## Actions`, `## 5 Whys`, `## Links`,
`## Lessons Learned`) are conventional and rendered by `pirs generate report`,
but the structured frontmatter fields are authoritative.

## JSON-PIR v1 schema

The `pirs export json` and `pirs import json` commands use a stable schema
documented at
[`schema/json-pir/v1.json`](https://github.com/joshrotenberg/pirs/blob/main/schema/json-pir/v1.json).
Redacted exports apply the patterns under `[privacy]` in `pirs.toml` to
sensitive fields before writing.

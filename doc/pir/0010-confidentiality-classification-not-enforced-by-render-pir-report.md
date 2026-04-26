---
number: 10
title: confidentiality classification not enforced by render_pir_report
status: Open
severity: Medium
incident_type: Process
problem_statement: |
  The pre-merge security review of feat/reports-metrics-language-audit
  (doc/reviews/feat-reports-metrics-language-audit-2026-04-26.md) flagged
  that pirs_core::report::render_pir_report does not consult
  pir.confidentiality before emitting full PIR text. A user invoking
  `pirs generate report <N>` against a PIR marked Restricted will get
  the full content with no warning or redaction, mirroring the gap that
  already exists in `pirs show`.
  This is a cross-command policy question: confidentiality enforcement
  should be uniform across show, generate report, generate actions, and
  the JSON-PIR export's --redact flag. Adding it inside REQ-RPT-001
  would create an inconsistent enforcement surface, so the finding is
  deferred and tracked here.
detected_at: 2026-04-26T00:00:00Z
people_involved:
  - name: GitHub Copilot
    type: agent
    role: pre-merge security reviewer
timeline:
  - at: 2026-04-26T00:00:00Z
    actor: GitHub Copilot
    type: detected
    description: Security subagent flagged FIND-001 during pre-merge review.
tags:
  - security-review
  - confidentiality
  - deferred
actions:
  - id: ACT-001
    description: Decide and document a uniform confidentiality enforcement policy across show, generate report, generate actions, and export (e.g. --include-confidential opt-in, or denial with override flag); record an ADR; implement consistently.
    owner: GitHub Copilot
    owner_type: agent
    status: Open
---

# PIR-0010 — Confidentiality classification not enforced by render_pir_report

## Problem Statement

See frontmatter.

## Impact

Information-disclosure risk for PIRs marked `Restricted` when an operator runs
`pirs generate report <N>` and shares the output. Existing `pirs show` has the
same shape; the new command does not regress the property but inherits the
gap.

## Timeline

See frontmatter.

## 5 Whys

1. Q: Why does `render_pir_report` print Restricted PIRs without a check?
   A: It mirrors the existing `pirs show` rendering loop, which has never
      enforced confidentiality.
2. Q: Why has `pirs show` never enforced confidentiality?
   A: Confidentiality has only been used by JSON-PIR export's `--redact`
      flag; no cross-command policy has been written.
3. Q: Why was that not added in this branch?
   A: The work is REQ-RPT-001 scope (Markdown report rendering); a uniform
      policy spans four commands and the export pipeline and warrants its
      own ADR.

## Actions

See frontmatter.

## Lessons Learned

- Pre-merge security review surfaces cross-command policy gaps the local
  feature scope cannot resolve cleanly.
- Confidentiality is currently informational metadata, not an enforcement
  primitive; treat enforcement as a deliberate cross-cutting feature.

## Links

- doc/reviews/feat-reports-metrics-language-audit-2026-04-26.md
- doc/adr/0008-keep-reports-metrics-and-language-audit-in-pirs-core.md

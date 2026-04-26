---
number: 1
title: 'Failing pytest run after dependency bump'
status: Resolved
severity: Low
incident_type: Development
problem_statement: |
  After bumping `requests` from 2.31 to 2.32 the unit-test suite started
  failing in CI with `TypeError: 'NoneType' object is not subscriptable`
  inside the HTTP retry helper. Local runs reproduced after a clean
  virtualenv rebuild.
occurred_at: 2026-04-20T09:12:00Z
detected_at: 2026-04-20T09:14:30Z
resolved_at: 2026-04-20T10:01:00Z
detection_method: CI
people_involved:
  - name: GitHub Copilot
    type: agent
    role: investigator
timeline:
  - at: 2026-04-20T09:14:30Z
    actor: GitHub Actions
    type: detected
    description: pytest job exited with code 1 on the `deps-bump` branch.
  - at: 2026-04-20T09:21:05Z
    actor: GitHub Copilot
    type: investigated
    description: |
      Reproduced locally; `requests==2.32` returns `None` from
      `HTTPAdapter.proxy_manager_for` when no proxy is configured, but
      the helper assumed a dict. Synthetic id `EXAMPLE_TOKEN_AAAA`
      surfaced in fixtures unchanged.
  - at: 2026-04-20T09:55:12Z
    actor: GitHub Copilot
    type: action_added
    description: Patched the helper to handle `None` and added a regression test.
  - at: 2026-04-20T10:01:00Z
    actor: pirs
    type: resolved
    description: status -> Resolved
five_whys:
  - question: Why did pytest fail?
    answer: HTTP retry helper indexed into `None`.
  - question: Why did the helper index into `None`?
    answer: The helper assumed `proxy_manager_for` always returned a dict.
  - question: Why was that assumption made?
    answer: It held for `requests` 2.31 and earlier; nobody re-checked on bump.
  - question: Why did the bump land without re-checking?
    answer: Dependency bumps were merged without exercising the retry path.
root_cause: Dependency bumps were merged without exercising the retry path.
actions:
  - id: A1
    description: Add a CI smoke test that exercises the retry helper end-to-end.
    owner: GitHub Copilot
    owner_type: agent
    status: Open
    due: 2026-05-15
links:
  - kind: PullRequest
    uri: https://example.invalid/repo/pull/42
    description: Bump requests to 2.32
  - kind: Commit
    uri: https://example.invalid/repo/commit/abc123
    description: Patch retry helper to handle None proxy_manager
tags:
  - dependencies
  - regression
  - python
confidentiality: Internal
---

# 1. Failing pytest run after dependency bump

> Type: development · Severity: low

## Problem Statement

After bumping `requests` from 2.31 to 2.32 the unit-test suite started
failing in CI with `TypeError: 'NoneType' object is not subscriptable`
inside the HTTP retry helper. Local runs reproduced after a clean
virtualenv rebuild.

## Impact

CI red on `deps-bump` branch for ~47 minutes. No production impact.

## People and Systems Involved

GitHub Actions (CI), GitHub Copilot (investigator agent).

## Timeline

See frontmatter; populated via `pirs timeline add`.

## Detection and Resolution Timing

Detected within 2.5 minutes of the failing job start; resolved 47 minutes later.

## 5 Whys

See frontmatter `five_whys`.

## Actions

See frontmatter `actions`.

## Lessons Learned

Dependency bumps must exercise non-default code paths. Synthetic
`EXAMPLE_TOKEN_AAAA` was used in fixtures; no real credentials touched.

## Links

See frontmatter `links`.

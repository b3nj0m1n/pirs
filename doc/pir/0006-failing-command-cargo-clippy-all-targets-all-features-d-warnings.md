---
number: 6
title: 'Failing command: cargo clippy --all-targets --all-features -- -D warnings'
status: Resolved
severity: Low
incident_type: Development
problem_statement: "Wrapped command exited with code 101.\n\nCommand: cargo clippy --all-targets --all-features -- -D warnings\n\nexit_code: 101\ncommand: cargo clippy --all-targets --all-features -- -D warnings\n--- stdout "
resolved_at: 2026-04-26T06:01:23.575613Z
timeline:
- at: 2026-04-26T06:01:23.553721Z
  actor: GitHub Copilot
  type: note
  description: Resolved by boxing JsonPirDocument variants; strict clippy now passes.
- at: 2026-04-26T06:01:23.575613Z
  actor: pirs
  type: resolved
  description: status -> Resolved
impact: _What systems, tests, environments, or workflows were affected?_
confidentiality: Internal
---

  --- stderr ---
      Checking pirs-core v0.1.0 (/Users/ben/IdeaProjects/pirs/crates/pirs-core)
  error: large size difference between variants
    --> crates/pirs-core/src/export.rs:45:1
     |
  45 | / pub enum JsonPirDocument {
  46 | |     Single(JsonPirSingle),
     | |     --------------------- the largest variant contains at least 680 bytes
  47 | |     Bulk(JsonPirBulkExport),
     | |     ----------------------- the second-largest variant contains at least 184 bytes
  48 | | }
     | |_^ the entire enum is at least 680 bytes
     |
     = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.92.0/index.html#large_enum_variant
     = note: `-D clippy::large-enum-variant` implied by `-D warnings`
     = help: to override `-D warnings` add `#[allow(clippy::large_enum_variant)]`
  help: consider boxing the large fields or introducing indirection in some other way to reduce the total size of the enum
     |
  46 -     Single(JsonPirSingle),
  46 +     Single(Box<JsonPirSingle>),
     |

  error: could not compile `pirs-core` (lib) due to 1 previous error
  warning: build failed, waiting for other jobs to finish...
  error: could not compile `pirs-core` (lib test) due to 1 previous error
occurred_at: 2026-04-26T06:00:15.887028Z
detected_at: 2026-04-26T06:00:18.836797Z
time_to_discover: PT2S
detection_method: agent-command-runner
people_involved:
- name: GitHub Copilot
  type: agent
timeline:
- at: 2026-04-26T06:00:18.836797Z
  actor: GitHub Copilot
  type: detected
  description: command failed (exit 101)
confidentiality: Internal
---

# 6. Failing command: cargo clippy --all-targets --all-features -- -D warnings

> Type: Development · Severity: Low

## Problem Statement

Wrapped command exited with code 101.

Command: cargo clippy --all-targets --all-features -- -D warnings

exit_code: 101
command: cargo clippy --all-targets --all-features -- -D warnings
--- stdout ---

--- stderr ---
    Checking pirs-core v0.1.0 (/Users/ben/IdeaProjects/pirs/crates/pirs-core)
error: large size difference between variants
  --> crates/pirs-core/src/export.rs:45:1
   |
45 | / pub enum JsonPirDocument {
46 | |     Single(JsonPirSingle),
   | |     --------------------- the largest variant contains at least 680 bytes
47 | |     Bulk(JsonPirBulkExport),
   | |     ----------------------- the second-largest variant contains at least 184 bytes
48 | | }
   | |_^ the entire enum is at least 680 bytes
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.92.0/index.html#large_enum_variant
   = note: `-D clippy::large-enum-variant` implied by `-D warnings`
   = help: to override `-D warnings` add `#[allow(clippy::large_enum_variant)]`
help: consider boxing the large fields or introducing indirection in some other way to reduce the total size of the enum
   |
46 -     Single(JsonPirSingle),
46 +     Single(Box<JsonPirSingle>),
   |

error: could not compile `pirs-core` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
error: could not compile `pirs-core` (lib test) due to 1 previous error


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



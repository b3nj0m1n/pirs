---
number: 14
title: 'Failing command: cargo clippy --all-targets --all-features -- -D warnings'
status: Resolved
severity: Low
incident_type: Development
problem_statement: "Wrapped command exited with code 101.\n\nCommand: cargo clippy --all-targets --all-features -- -D warnings\n\nexit_code: 101\ncommand: cargo clippy --all-targets --all-features -- -D warnings\n--- stdout "
resolved_at: 2026-04-26T08:20:27.913748Z
timeline:
- at: 2026-04-26T08:20:27.881923Z
  actor: GitHub Copilot
  type: investigated
  description: Clippy flagged a nested if in extract_numbers after avoiding let-chain syntax. Rewrote the tail as an early return plus a single parse check and reran clippy successfully.
- at: 2026-04-26T08:20:27.913748Z
  actor: pirs
  type: resolved
  description: status -> Resolved
five_whys:
- question: Why did clippy fail?
  answer: The no-let-chain workaround introduced a nested if that triggered clippy::collapsible_if under -D warnings.
impact: _What systems, tests, environments, or workflows were affected?_
root_cause: The no-let-chain workaround introduced a nested if that triggered clippy::collapsible_if under -D warnings.
confidentiality: Internal
---

  --- stderr ---
      Checking pirs-core v0.1.0 (/Users/ben/IdeaProjects/pirs/crates/pirs-core)
  error: this `if` statement can be collapsed
     --> crates/pirs-core/src/related.rs:274:5
      |
  274 | /     if !current.is_empty() {
  275 | |         if let Ok(number) = current.parse::<u32>() {
  276 | |             numbers.push(number);
  277 | |         }
  278 | |     }
      | |_____^
      |
      = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.92.0/index.html#collapsible_if
      = note: `-D clippy::collapsible-if` implied by `-D warnings`
      = help: to override `-D warnings` add `#[allow(clippy::collapsible_if)]`
  help: collapse nested if block
      |
  274 ~     if !current.is_empty()
  275 ~         && let Ok(number) = current.parse::<u32>() {
  276 |             numbers.push(number);
  277 ~         }
      |

  error: could not compile `pirs-core` (lib) due to 1 previous error
  warning: build failed, waiting for other jobs to finish...
  error: could not compile `pirs-core` (lib test) due to 1 previous error
occurred_at: 2026-04-26T08:19:36.89949Z
detected_at: 2026-04-26T08:19:41.494142Z
time_to_discover: PT4S
detection_method: agent-command-runner
people_involved:
- name: GitHub Copilot
  type: agent
timeline:
- at: 2026-04-26T08:19:41.494142Z
  actor: GitHub Copilot
  type: detected
  description: command failed (exit 101)
confidentiality: Internal
---

# 14. Failing command: cargo clippy --all-targets --all-features -- -D warnings

> Type: Development · Severity: Low

## Problem Statement

Wrapped command exited with code 101.

Command: cargo clippy --all-targets --all-features -- -D warnings

exit_code: 101
command: cargo clippy --all-targets --all-features -- -D warnings
--- stdout ---

--- stderr ---
    Checking pirs-core v0.1.0 (/Users/ben/IdeaProjects/pirs/crates/pirs-core)
error: this `if` statement can be collapsed
   --> crates/pirs-core/src/related.rs:274:5
    |
274 | /     if !current.is_empty() {
275 | |         if let Ok(number) = current.parse::<u32>() {
276 | |             numbers.push(number);
277 | |         }
278 | |     }
    | |_____^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.92.0/index.html#collapsible_if
    = note: `-D clippy::collapsible-if` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::collapsible_if)]`
help: collapse nested if block
    |
274 ~     if !current.is_empty()
275 ~         && let Ok(number) = current.parse::<u32>() {
276 |             numbers.push(number);
277 ~         }
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





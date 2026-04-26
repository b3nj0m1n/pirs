---
number: 7
title: 'Failing command: cargo clippy --all-targets --all-features -- -D warnings'
status: Resolved
severity: Low
incident_type: Development
problem_statement: "Wrapped command exited with code 101.\n\nCommand: cargo clippy --all-targets --all-features -- -D warnings\n\nexit_code: 101\ncommand: cargo clippy --all-targets --all-features -- -D warnings\n--- stdout "
resolved_at: 2026-04-26T06:01:23.97028Z
timeline:
- at: 2026-04-26T06:01:23.949692Z
  actor: GitHub Copilot
  type: note
  description: Resolved by applying mechanical clippy fixes in new.rs and run_cmd.rs; strict clippy now passes.
- at: 2026-04-26T06:01:23.97028Z
  actor: pirs
  type: resolved
  description: status -> Resolved
impact: _What systems, tests, environments, or workflows were affected?_
confidentiality: Internal
---

  --- stderr ---
      Checking pirs-core v0.1.0 (/Users/ben/IdeaProjects/pirs/crates/pirs-core)
      Checking pirs v0.1.0 (/Users/ben/IdeaProjects/pirs/crates/pirs)
  error: redundant closure
    --> crates/pirs/src/commands/new.rs:70:29
     |
  70 |             .unwrap_or_else(|| whoami::username());
     |                             ^^^^^^^^^^^^^^^^^^^^^ help: replace the closure with the function itself: `whoami::username`
     |
     = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.92.0/index.html#redundant_closure
     = note: `-D clippy::redundant-closure` implied by `-D warnings`
     = help: to override `-D warnings` add `#[allow(clippy::redundant_closure)]`

  error: this `if` statement can be collapsed
    --> crates/pirs/src/commands/new.rs:82:5
     |
  82 | /     if !args.no_edit && atty_stdin() {
  83 | |         if let Err(e) = edit::edit_file(&path) {
  84 | |             eprintln!("warning: could not open editor: {e}");
  85 | |         }
  86 | |     }
     | |_____^
     |
     = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.92.0/index.html#collapsible_if
     = note: `-D clippy::collapsible-if` implied by `-D warnings`
     = help: to override `-D warnings` add `#[allow(clippy::collapsible_if)]`
  help: collapse nested if block
     |
  82 ~     if !args.no_edit && atty_stdin()
  83 ~         && let Err(e) = edit::edit_file(&path) {
  84 |             eprintln!("warning: could not open editor: {e}");
  85 ~         }
     |

  error: redundant closure
    --> crates/pirs/src/commands/run_cmd.rs:63:51
     |
  63 |     let actor = args.agent.clone().unwrap_or_else(|| whoami::username());
     |                                                   ^^^^^^^^^^^^^^^^^^^^^ help: replace the closure with the function itself: `whoami::username`
     |
     = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.92.0/index.html#redundant_closure

  error: redundant closure
     --> crates/pirs/src/commands/run_cmd.rs:102:56
      |
  102 |     let actor_name = args.agent.clone().unwrap_or_else(|| whoami::username());
      |                                                        ^^^^^^^^^^^^^^^^^^^^^ help: replace the closure with the function itself: `whoami::username`
      |
      = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.92.0/index.html#redundant_closure

  error: could not compile `pirs` (bin "pirs" test) due to 4 previous errors
  warning: build failed, waiting for other jobs to finish...
  error: could not compile `pirs` (bin "pirs") due to 4 previous errors
occurred_at: 2026-04-26T06:00:36.992899Z
detected_at: 2026-04-26T06:00:42.187577Z
time_to_discover: PT5S
detection_method: agent-command-runner
people_involved:
- name: GitHub Copilot
  type: agent
timeline:
- at: 2026-04-26T06:00:42.187577Z
  actor: GitHub Copilot
  type: detected
  description: command failed (exit 101)
confidentiality: Internal
---

# 7. Failing command: cargo clippy --all-targets --all-features -- -D warnings

> Type: Development · Severity: Low

## Problem Statement

Wrapped command exited with code 101.

Command: cargo clippy --all-targets --all-features -- -D warnings

exit_code: 101
command: cargo clippy --all-targets --all-features -- -D warnings
--- stdout ---

--- stderr ---
    Checking pirs-core v0.1.0 (/Users/ben/IdeaProjects/pirs/crates/pirs-core)
    Checking pirs v0.1.0 (/Users/ben/IdeaProjects/pirs/crates/pirs)
error: redundant closure
  --> crates/pirs/src/commands/new.rs:70:29
   |
70 |             .unwrap_or_else(|| whoami::username());
   |                             ^^^^^^^^^^^^^^^^^^^^^ help: replace the closure with the function itself: `whoami::username`
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.92.0/index.html#redundant_closure
   = note: `-D clippy::redundant-closure` implied by `-D warnings`
   = help: to override `-D warnings` add `#[allow(clippy::redundant_closure)]`

error: this `if` statement can be collapsed
  --> crates/pirs/src/commands/new.rs:82:5
   |
82 | /     if !args.no_edit && atty_stdin() {
83 | |         if let Err(e) = edit::edit_file(&path) {
84 | |             eprintln!("warning: could not open editor: {e}");
85 | |         }
86 | |     }
   | |_____^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.92.0/index.html#collapsible_if
   = note: `-D clippy::collapsible-if` implied by `-D warnings`
   = help: to override `-D warnings` add `#[allow(clippy::collapsible_if)]`
help: collapse nested if block
   |
82 ~     if !args.no_edit && atty_stdin()
83 ~         && let Err(e) = edit::edit_file(&path) {
84 |             eprintln!("warning: could not open editor: {e}");
85 ~         }
   |

error: redundant closure
  --> crates/pirs/src/commands/run_cmd.rs:63:51
   |
63 |     let actor = args.agent.clone().unwrap_or_else(|| whoami::username());
   |                                                   ^^^^^^^^^^^^^^^^^^^^^ help: replace the closure with the function itself: `whoami::username`
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.92.0/index.html#redundant_closure

error: redundant closure
   --> crates/pirs/src/commands/run_cmd.rs:102:56
    |
102 |     let actor_name = args.agent.clone().unwrap_or_else(|| whoami::username());
    |                                                        ^^^^^^^^^^^^^^^^^^^^^ help: replace the closure with the function itself: `whoami::username`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.92.0/index.html#redundant_closure

error: could not compile `pirs` (bin "pirs" test) due to 4 previous errors
warning: build failed, waiting for other jobs to finish...
error: could not compile `pirs` (bin "pirs") due to 4 previous errors


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



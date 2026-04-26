---
number: 12
title: 'Failing command: rustfmt crates/pirs-core/src/related.rs crates/pirs-core/src/lib.rs crates/pirs/src/mcp.rs crates/pirs-core/tests/related.rs crates/pirs/tests/mcp.rs'
status: Resolved
severity: Low
incident_type: Development
problem_statement: "Wrapped command exited with code 1.\n\nCommand: rustfmt crates/pirs-core/src/related.rs crates/pirs-core/src/lib.rs crates/pirs/src/mcp.rs crates/pirs-core/tests/related.rs crates/pirs/tests/mcp.rs\n\nexit_code: 1\ncommand: rustfmt crates/pirs-core/src/related.rs crates/pirs-core/src/lib.rs crates/pirs/src/mcp.rs crates/pirs-core/tests/related.rs crates/pirs/tests/mcp.rs\n--- stdout "
resolved_at: 2026-04-26T08:16:51.123728Z
timeline:
- at: 2026-04-26T08:16:51.085076Z
  actor: GitHub Copilot
  type: investigated
  description: Direct rustfmt failed because it did not infer the workspace edition and because related.rs used let-chain syntax. Replaced the let-chain and reran rustfmt with --edition 2024 successfully.
- at: 2026-04-26T08:16:51.123728Z
  actor: pirs
  type: resolved
  description: status -> Resolved
five_whys:
- question: Why did direct rustfmt fail?
  answer: The direct rustfmt invocation lacked Cargo's edition context, and the new helper used syntax that older parser modes rejected.
impact: _What systems, tests, environments, or workflows were affected?_
root_cause: The direct rustfmt invocation lacked Cargo's edition context, and the new helper used syntax that older parser modes rejected.
confidentiality: Internal
---

  --- stderr ---
  error: let chains are only allowed in Rust 2024 or later
     --> /Users/ben/IdeaProjects/pirs/crates/pirs-core/src/related.rs:274:12
      |
  274 |         && let Ok(number) = current.parse::<u32>()
      |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

  error: let chains are only allowed in Rust 2024 or later
     --> /Users/ben/IdeaProjects/pirs/crates/pirs-core/src/config.rs:177:8
      |
  177 |     if let Ok(dir) = std::env::var(ENV_PIR_DIRECTORY)
      |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

  Error writing files: failed to resolve mod `config`: cannot parse /Users/ben/IdeaProjects/pirs/crates/pirs-core/src/config.rs
  error: `async move` blocks are only allowed in Rust 2018 or later
    --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:66:22
     |
  66 |     runtime.block_on(async move { run_async(state, http_addr).await })
     |                      ^^^^^^^^^^

  error[E0670]: `async fn` is not permitted in Rust 2015
    --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:69:1
     |
  69 | async fn run_async(state: PirState, http_addr: Option<String>) -> Result<()> {
     | ^^^^^ to use `async fn`, switch to Rust 2018 or later
     |
     = help: pass `--edition 2024` to `rustc`
     = note: for more on editions, read https://doc.rust-lang.org/edition-guide

  error[E0670]: `async fn` is not permitted in Rust 2015
    --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:83:1
     |
  83 | async fn run_http(router: McpRouter, addr: String) -> Result<()> {
     | ^^^^^ to use `async fn`, switch to Rust 2018 or later
     |
     = help: pass `--edition 2024` to `rustc`
     = note: for more on editions, read https://doc.rust-lang.org/edition-guide

  error[E0670]: `async fn` is not permitted in Rust 2015
     --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:100:1
      |
  100 | async fn run_http(_router: McpRouter, _addr: String) -> Result<()> {
      | ^^^^^ to use `async fn`, switch to Rust 2018 or later
      |
      = help: pass `--edition 2024` to `rustc`
      = note: for more on editions, read https://doc.rust-lang.org/edition-guide

  error: `async move` blocks are only allowed in Rust 2018 or later
     --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:273:81
      |
  273 |             |State(st): State<Arc<PirState>>, Json(input): Json<ListPirsInput>| async move {
      |                                                                                 ^^^^^^^^^^

  error: `async move` blocks are only allowed in Rust 2018 or later
     --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:305:79
      |
  305 |             |State(st): State<Arc<PirState>>, Json(input): Json<GetPirInput>| async move {
      |                                                                               ^^^^^^^^^^

  error: `async move` blocks are only allowed in Rust 2018 or later
     --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:336:83
      |
  336 |             |State(st): State<Arc<PirState>>, Json(input): Json<SearchPirsInput>| async move {
      |                                                                                   ^^^^^^^^^^

  error: `async move` blocks are only allowed in Rust 2018 or later
     --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:385:84
      |
  385 |             |State(st): State<Arc<PirState>>, Json(input): Json<OpenActionsInput>| async move {
      |                                                                                    ^^^^^^^^^^

  error: `async move` blocks are only allowed in Rust 2018 or later
     --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:442:96
      |
  442 |         .extractor_handler(state, |State(st): State<Arc<PirState>>, Json(_): Json<EmptyInput>| async move {
      |                                                                                                ^^^^^^^^^^

  error: `async move` blocks are only allowed in Rust 2018 or later
     --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:478:84
      |
  478 |             |State(st): State<Arc<PirState>>, Json(input): Json<ValidatePirInput>| async move {
      |                                                                                    ^^^^^^^^^^

  error: `async move` blocks are only allowed in Rust 2018 or later
     --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:538:88
      |
  538 |             |State(st): State<Arc<PirState>>, Json(input): Json<IncidentMetricsInput>| async move {
      |                                                                                        ^^^^^^^^^^

  error: `async move` blocks are only allowed in Rust 2018 or later
     --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:602:91
      |
  602 |             |State(st): State<Arc<PirState>>, Json(input): Json<SuggestRelatedPirsInput>| async move {
      |                                                                                           ^^^^^^^^^^

  error: `async move` blocks are only allowed in Rust 2018 or later
     --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:660:82
      |
  660 |             |State(st): State<Arc<PirState>>, Json(input): Json<CreatePirInput>| async move {
      |                                                                                  ^^^^^^^^^^

  error: `async move` blocks are only allowed in Rust 2018 or later
     --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:745:87
      |
  745 |             |State(st): State<Arc<PirState>>, Json(input): Json<AppendTimelineInput>| async move {
      |                                                                                       ^^^^^^^^^^

  error: `async move` blocks are only allowed in Rust 2018 or later
     --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:792:85
      |
  792 |             |State(st): State<Arc<PirState>>, Json(input): Json<UpdateStatusInput>| async move {
      |                                                                                     ^^^^^^^^^^

  error: `async move` blocks are only allowed in Rust 2018 or later
     --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:833:79
      |
  833 |             |State(st): State<Arc<PirState>>, Json(input): Json<AddWhyInput>| async move {
      |                                                                               ^^^^^^^^^^

  error: `async move` blocks are only allowed in Rust 2018 or later
     --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:882:82
      |
  882 |             |State(st): State<Arc<PirState>>, Json(input): Json<AddActionInput>| async move {
      |                                                                                  ^^^^^^^^^^

  error: `async move` blocks are only allowed in Rust 2018 or later
     --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:927:85
      |
  927 |             |State(st): State<Arc<PirState>>, Json(input): Json<UpdateActionInput>| async move {
      |                                                                                     ^^^^^^^^^^

  error: `async move` blocks are only allowed in Rust 2018 or later
     --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:968:85
      |
  968 |             |State(st): State<Arc<PirState>>, Json(input): Json<LinkEvidenceInput>| async move {
      |                                                                                     ^^^^^^^^^^

occurred_at: 2026-04-26T08:15:53.69603Z
detected_at: 2026-04-26T08:15:53.950022Z
time_to_discover: PT0S
detection_method: agent-command-runner
people_involved:
- name: GitHub Copilot
  type: agent
timeline:
- at: 2026-04-26T08:15:53.950022Z
  actor: GitHub Copilot
  type: detected
  description: command failed (exit 1)
confidentiality: Internal
---

# 12. Failing command: rustfmt crates/pirs-core/src/related.rs crates/pirs-core/src/lib.rs crates/pirs/src/mcp.rs crates/pirs-core/tests/related.rs crates/pirs/tests/mcp.rs

> Type: Development · Severity: Low

## Problem Statement

Wrapped command exited with code 1.

Command: rustfmt crates/pirs-core/src/related.rs crates/pirs-core/src/lib.rs crates/pirs/src/mcp.rs crates/pirs-core/tests/related.rs crates/pirs/tests/mcp.rs

exit_code: 1
command: rustfmt crates/pirs-core/src/related.rs crates/pirs-core/src/lib.rs crates/pirs/src/mcp.rs crates/pirs-core/tests/related.rs crates/pirs/tests/mcp.rs
--- stdout ---

--- stderr ---
error: let chains are only allowed in Rust 2024 or later
   --> /Users/ben/IdeaProjects/pirs/crates/pirs-core/src/related.rs:274:12
    |
274 |         && let Ok(number) = current.parse::<u32>()
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error: let chains are only allowed in Rust 2024 or later
   --> /Users/ben/IdeaProjects/pirs/crates/pirs-core/src/config.rs:177:8
    |
177 |     if let Ok(dir) = std::env::var(ENV_PIR_DIRECTORY)
    |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Error writing files: failed to resolve mod `config`: cannot parse /Users/ben/IdeaProjects/pirs/crates/pirs-core/src/config.rs
error: `async move` blocks are only allowed in Rust 2018 or later
  --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:66:22
   |
66 |     runtime.block_on(async move { run_async(state, http_addr).await })
   |                      ^^^^^^^^^^

error[E0670]: `async fn` is not permitted in Rust 2015
  --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:69:1
   |
69 | async fn run_async(state: PirState, http_addr: Option<String>) -> Result<()> {
   | ^^^^^ to use `async fn`, switch to Rust 2018 or later
   |
   = help: pass `--edition 2024` to `rustc`
   = note: for more on editions, read https://doc.rust-lang.org/edition-guide

error[E0670]: `async fn` is not permitted in Rust 2015
  --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:83:1
   |
83 | async fn run_http(router: McpRouter, addr: String) -> Result<()> {
   | ^^^^^ to use `async fn`, switch to Rust 2018 or later
   |
   = help: pass `--edition 2024` to `rustc`
   = note: for more on editions, read https://doc.rust-lang.org/edition-guide

error[E0670]: `async fn` is not permitted in Rust 2015
   --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:100:1
    |
100 | async fn run_http(_router: McpRouter, _addr: String) -> Result<()> {
    | ^^^^^ to use `async fn`, switch to Rust 2018 or later
    |
    = help: pass `--edition 2024` to `rustc`
    = note: for more on editions, read https://doc.rust-lang.org/edition-guide

error: `async move` blocks are only allowed in Rust 2018 or later
   --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:273:81
    |
273 |             |State(st): State<Arc<PirState>>, Json(input): Json<ListPirsInput>| async move {
    |                                                                                 ^^^^^^^^^^

error: `async move` blocks are only allowed in Rust 2018 or later
   --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:305:79
    |
305 |             |State(st): State<Arc<PirState>>, Json(input): Json<GetPirInput>| async move {
    |                                                                               ^^^^^^^^^^

error: `async move` blocks are only allowed in Rust 2018 or later
   --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:336:83
    |
336 |             |State(st): State<Arc<PirState>>, Json(input): Json<SearchPirsInput>| async move {
    |                                                                                   ^^^^^^^^^^

error: `async move` blocks are only allowed in Rust 2018 or later
   --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:385:84
    |
385 |             |State(st): State<Arc<PirState>>, Json(input): Json<OpenActionsInput>| async move {
    |                                                                                    ^^^^^^^^^^

error: `async move` blocks are only allowed in Rust 2018 or later
   --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:442:96
    |
442 |         .extractor_handler(state, |State(st): State<Arc<PirState>>, Json(_): Json<EmptyInput>| async move {
    |                                                                                                ^^^^^^^^^^

error: `async move` blocks are only allowed in Rust 2018 or later
   --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:478:84
    |
478 |             |State(st): State<Arc<PirState>>, Json(input): Json<ValidatePirInput>| async move {
    |                                                                                    ^^^^^^^^^^

error: `async move` blocks are only allowed in Rust 2018 or later
   --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:538:88
    |
538 |             |State(st): State<Arc<PirState>>, Json(input): Json<IncidentMetricsInput>| async move {
    |                                                                                        ^^^^^^^^^^

error: `async move` blocks are only allowed in Rust 2018 or later
   --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:602:91
    |
602 |             |State(st): State<Arc<PirState>>, Json(input): Json<SuggestRelatedPirsInput>| async move {
    |                                                                                           ^^^^^^^^^^

error: `async move` blocks are only allowed in Rust 2018 or later
   --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:660:82
    |
660 |             |State(st): State<Arc<PirState>>, Json(input): Json<CreatePirInput>| async move {
    |                                                                                  ^^^^^^^^^^

error: `async move` blocks are only allowed in Rust 2018 or later
   --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:745:87
    |
745 |             |State(st): State<Arc<PirState>>, Json(input): Json<AppendTimelineInput>| async move {
    |                                                                                       ^^^^^^^^^^

error: `async move` blocks are only allowed in Rust 2018 or later
   --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:792:85
    |
792 |             |State(st): State<Arc<PirState>>, Json(input): Json<UpdateStatusInput>| async move {
    |                                                                                     ^^^^^^^^^^

error: `async move` blocks are only allowed in Rust 2018 or later
   --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:833:79
    |
833 |             |State(st): State<Arc<PirState>>, Json(input): Json<AddWhyInput>| async move {
    |                                                                               ^^^^^^^^^^

error: `async move` blocks are only allowed in Rust 2018 or later
   --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:882:82
    |
882 |             |State(st): State<Arc<PirState>>, Json(input): Json<AddActionInput>| async move {
    |                                                                                  ^^^^^^^^^^

error: `async move` blocks are only allowed in Rust 2018 or later
   --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:927:85
    |
927 |             |State(st): State<Arc<PirState>>, Json(input): Json<UpdateActionInput>| async move {
    |                                                                                     ^^^^^^^^^^

error: `async move` blocks are only allowed in Rust 2018 or later
   --> /Users/ben/IdeaProjects/pirs/crates/pirs/src/mcp.rs:968:85
    |
968 |             |State(st): State<Arc<PirState>>, Json(input): Json<LinkEvidenceInput>| async move {
    |                                                                                     ^^^^^^^^^^



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





---
number: 9
title: Generate shell completions via clap_complete subcommand
status: accepted
date: 2026-04-26
tags:
  - reversible-easy
  - cli
  - completions
---

# Generate shell completions via clap_complete subcommand

## Context and Problem Statement

The bootstrap roadmap (spec §9.2) lists "Add shell completions" as
unfinished work. `clap_complete = "4"` is already a workspace dependency,
so the engineering question is *how* completions are exposed: as a
runtime subcommand the user invokes (`pirs completions <shell>`), or as
build-time artefacts emitted by a `build.rs` into `OUT_DIR` and shipped
alongside the binary.

This decision is small but it shapes the packaging story for downstream
distributors and the discoverability story for LLM agents reading
`pirs --help`.

## Decision Drivers

* Discoverability for LLM agents and humans — `pirs --help` should make
  the capability visible, mirroring REQ-DOC-007 (agent workflow docs).
* Packaging: distributors (Homebrew, deb, AUR) need a deterministic way
  to vendor completion files at install time.
* Match `adrs` precedent (the parent project) which uses a runtime
  subcommand.
* Avoid adding a build-script dependency or build complexity for a
  one-time output.

## Considered Options

* **A. Runtime `pirs completions <shell> [--out-dir DIR]` subcommand**
  using `clap_complete::generate` / `generate_to`. Default writes the
  script to stdout; `--out-dir` writes the canonical filename inside a
  directory.
* **B. Build-time generation via `build.rs`** that emits scripts into
  `OUT_DIR` and an installer script copies them to the right system
  location.

## Decision Outcome

Chosen option: **A — runtime subcommand**.

It mirrors `adrs`, keeps the build pipeline unchanged, and surfaces the
feature through `pirs --help` so both packagers and LLM agents can
discover it. `--out-dir` covers the packager use case without forcing a
build script.

### Consequences

* Good — single canonical entry point usable from packaging scripts and
  interactive shells alike.
* Good — no `build.rs`, no widening of the build-time dependency
  surface.
* Good — testable through standard `assert_cmd` integration tests.
* Bad — packagers must invoke the binary post-build to emit files, a
  trivial extra step compared with shipping pre-rendered artefacts.

### Confirmation

CLI integration test asserts `pirs completions bash` exits 0 and emits a
non-empty completion script; covered by REQ-COMP-001..003 acceptance
criteria.

## Pros and Cons of the Options

### A. Runtime subcommand

* Good, because consistent with `adrs` and `cargo` itself
  (`cargo completions` via `clap_complete`).
* Good, because keeps build deterministic and free of script-side
  effects.
* Good, because `--help` advertises the feature.
* Neutral, because packagers must call the binary once during install.

### B. Build-time generation via `build.rs`

* Good, because pre-rendered scripts are immediately available without
  invoking the binary.
* Bad, because adds a build-time path that is exercised on every
  release build for a feature most users never touch.
* Bad, because hidden from users; not visible in `pirs --help`.
* Bad, because no parity with `adrs`.

## More Information

Implements spec §9.2 ("Add shell completions"). Reversibility is easy:
the subcommand can be removed and replaced by `build.rs` artefacts in a
single PR if a future packaging workflow requires it.

---
number: 10
title: Use mdBook with hand-written chapters for user docs
status: accepted
date: 2026-04-26
tags:
  - reversible-easy
  - docs
  - mdbook
---

# Use mdBook with hand-written chapters for user docs

## Context and Problem Statement

Spec §9.7 defines five documentation deliverables (REQ-DOC-005..009):
getting-started guide, PIR file format, agent workflow patterns,
incident severity / status taxonomy, and MCP security expectations. The
spec §3.2 workspace layout reserves `book/` for "optional mdBook
documentation". The project README and the requirements spec serve
different audiences (humans skimming, build target) and neither is
suitable as task-oriented user documentation.

The decision is whether to publish docs as one long Markdown file, as a
mdBook with hand-written chapters, or as auto-generated CLI reference
from `clap`.

## Decision Drivers

* Match the parent project (`adrs`) precedent, which ships an mdBook.
* Provide sidebar navigation, search, and chapter-level deep links for
  both humans and LLM agents reading specific sections.
* Keep the writing surface small enough that a solo maintainer can keep
  it accurate.
* Avoid build-time coupling that breaks `cargo build`.

## Considered Options

* **A. mdBook under `book/`** with hand-written chapters mapped to
  REQ-DOC-005..009.
* **B. Single long `docs/USAGE.md`** committed at repo root.
* **C. Auto-generated CLI reference from clap** (e.g. `clap-markdown`)
  as the primary documentation surface.

## Decision Outcome

Chosen option: **A — mdBook with hand-written chapters**.

mdBook gives sidebar navigation and search out of the box, matches the
spec §3.2 layout, and matches `adrs` precedent. Chapters are scoped 1:1
to REQ-DOC-005..009 so each requirement has a single owning file.
Auto-generated CLI reference is left as future work — `--help` already
covers that surface.

### Consequences

* Good — clear chapter-to-requirement mapping; easy to audit coverage.
* Good — search and deep links help LLM agents fetch specific guidance
  (e.g. "MCP security expectations") without scanning a long file.
* Good — `book/` is decoupled from `cargo build`; broken docs do not
  break the binary build.
* Bad — adds an out-of-tree build step (`mdbook build book`) that is
  not exercised by `cargo test`.
* Neutral — drift risk between docs and behaviour; mitigated by linking
  chapters to spec requirement IDs.

### Confirmation

`mdbook build book` exits 0 with no warnings on a clean checkout
(REQ-DOC-BOOK-001). `book/src/SUMMARY.md` references one chapter file
per REQ-DOC-005..009 (REQ-DOC-BOOK-002).

## Pros and Cons of the Options

### A. mdBook

* Good, because sidebar + search aid both humans and LLM agents.
* Good, because matches `adrs` precedent.
* Good, because spec §3.2 already reserves `book/`.
* Bad, because requires a separate `mdbook` toolchain install for
  contributors who want to render locally.

### B. Single docs/USAGE.md

* Good, because zero new tooling.
* Bad, because no search or navigation; all five REQ-DOC topics
  collapse into one document.
* Bad, because does not scale as the surface grows.

### C. Auto-generated CLI reference

* Good, because zero drift between code and reference.
* Bad, because mechanical reference is not the same artefact as a
  getting-started guide, file-format reference, or security guidance.
* Bad, because does not satisfy REQ-DOC-005..009 on its own.

## More Information

Implements spec §9.7. CI publishing of the rendered book and an
mdbook-lint preprocessor are explicit non-goals for this iteration.
Reversibility is easy: the `book/` directory can be deleted or
collapsed into a single file with no impact on the binary.

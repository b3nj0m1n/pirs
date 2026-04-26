---
number: 11
title: Hand-curated PIR fixture corpus under tests/fixtures/pir-corpus
status: accepted
date: 2026-04-26
tags:
  - reversible-easy
  - testing
  - fixtures
---

# Hand-curated PIR fixture corpus under tests/fixtures/pir-corpus

## Context and Problem Statement

Spec §9.6 lists a fixture corpus covering development, production,
security, and process PIRs as outstanding work. Existing tests construct
PIRs inline via `tempfile`-backed repositories or via `pirs new`
invocations, which is sufficient for behaviour tests but provides no
canonical, reviewable examples of the on-disk format.

The decision is where the corpus lives, what shape it takes (static
files vs. generators), and how it is validated to stay in sync with the
parser.

## Decision Drivers

* Static example files double as documentation: they show authors what
  a "good" PIR looks like for each incident type.
* Tests must catch parser drift: any change to YAML frontmatter or
  Markdown section semantics that breaks an example surfaces in CI.
* Fixture content must not embed real secrets; redaction policy in
  `pirs-core` covers runtime data, but committed fixtures need a
  separate guard.
* Agent users (LLMs) benefit from canonical examples they can read
  before generating their own PIRs via the MCP `create_pir` tool.

## Considered Options

* **A. Static files in `tests/fixtures/pir-corpus/<type>/`** — one
  canonical PIR per type, parsed by a smoke test that walks the
  directory. Synthetic-only identifiers (`EXAMPLE_TOKEN_AAAA`) and a
  CI grep guard reject common secret prefixes.
* **B. Inline fixtures inside test files** via raw string literals or
  programmatic construction. No filesystem corpus.
* **C. Property-based generators** (`proptest`) producing random PIRs
  per type at test time.

## Decision Outcome

Chosen option: **A — static files**.

Static fixtures are reviewable in pull requests, double as
documentation, and the smoke test (`crates/pirs-core/tests/fixtures_parse.rs`)
catches parser drift on every CI run. They live under `tests/fixtures/`
so they are not picked up by config discovery when running `pirs` against
the workspace root (`doc/pir/` is the default repository location).

### Consequences

* Good — corpus content is reviewable and serves as living
  documentation for each incident type.
* Good — smoke test guarantees the corpus parses; any breaking parser
  change surfaces in CI before release.
* Good — corpus directory is outside `doc/pir/` so it does not pollute
  the project's own PIR repository when running `pirs list` from the
  workspace root.
* Bad — static fixtures can drift from the schema if maintainers forget
  to update them; mitigated by the parse smoke test.
* Bad — narrower coverage than generators would provide.

### Confirmation

`crates/pirs-core/tests/fixtures_parse.rs` walks the corpus and
asserts `pirs_core::parse::parse_pir` returns `Ok` for every file
(REQ-FIX-001). At least one PIR per type {`development`,
`production`, `security`, `process`} is present (REQ-FIX-002). A
secret-prefix grep guard runs as part of the same test (REQ-FIX-003).

## Pros and Cons of the Options

### A. Static files

* Good, because reviewable in PRs and usable as documentation examples.
* Good, because the smoke test gives a deterministic, fast regression
  signal.
* Neutral, because requires manual updates when the schema evolves.

### B. Inline fixtures

* Good, because no filesystem coupling for tests.
* Bad, because not visible to authors as "what good looks like".
* Bad, because each test grows its own fixture, encouraging duplication
  and drift.

### C. Property-based generators

* Good, because broad input coverage.
* Bad, because random outputs are not human-readable; they do not
  document the type.
* Bad, because generators themselves must be maintained against schema
  changes — moves the drift problem rather than solving it.
* Out of scope for this iteration; left as future work.

## More Information

Implements spec §9.6 fixture-corpus deliverable. Reversibility is easy:
fixtures can be deleted or replaced by inline construction in a single
PR. Property-based generators may be layered on top in a future ADR
without invalidating this one.

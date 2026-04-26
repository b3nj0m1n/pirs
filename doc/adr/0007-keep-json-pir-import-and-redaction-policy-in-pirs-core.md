---
number: 7
title: Keep JSON-PIR import and redaction policy in pirs-core
status: accepted
date: 2026-04-26
tags:
  - reversible-easy
  - json-pir
  - redaction
  - import
---

# Keep JSON-PIR import and redaction policy in pirs-core

## Context and Problem Statement

`pirs` needs to implement JSON-PIR import and redacted JSON export. The
feature touches both the command-line surface and reusable data
transformations: importing must parse single and bulk JSON-PIR v1 documents,
plan number-collision behavior, and write Markdown PIR files, while redacted
export must apply repository privacy configuration before JSON reaches stdout.

The design question is where the JSON-PIR policy should live. Keeping policy
in command modules is quick, but it risks duplicating behavior if MCP tools or
library users later need the same import/redaction semantics. Moving the pure
parts into `pirs-core` keeps CLI code focused on I/O and makes the behavior
easier to test without filesystem setup.

## Decision Drivers

* Preserve the CLI as an imperative shell for argument parsing, file/stdin
  reads, stdout/stderr reporting, and repository writes.
* Keep JSON-PIR parsing, schema-version checks, redaction, and import planning
  deterministic enough to test directly in `pirs-core`.
* Reuse the existing `Repository` type as the only writer of Markdown PIR
  files.
* Make the implementation easy to reuse from future MCP tools without copying
  command-module logic.
* Keep the change reversible and small: no new storage format, dependency
  service, or public protocol version is introduced.

## Considered Options

* Put JSON-PIR parsing, redaction, and import planning in `pirs-core`, with
  command modules handling only I/O and reporting.
* Keep most import and redaction behavior in CLI command modules, with only
  minimal helper functions in `pirs-core`.

## Decision Outcome

Chosen option: **"Put JSON-PIR parsing, redaction, and import planning in
`pirs-core`, with command modules handling only I/O and reporting"**, because
the policy is part of the PIR data model rather than the terminal UI. The CLI
still owns side effects: it reads files or stdin, opens the repository, prints
dry-run/import summaries, and calls repository write operations. Core helpers
own pure transformations such as parsing JSON-PIR v1 envelopes, validating the
schema version, applying redaction patterns to JSON values, recomputing derived
durations, and classifying imports as new, skipped, or overwrite candidates.

### Consequences

* Good, because the CLI command modules remain thin and easy to inspect.
* Good, because redaction and import behavior can be unit-tested without
  invoking a subprocess.
* Good, because future MCP import/export tools can reuse the same core policy.
* Good, because the design is reversible-easy: moving helpers back into command
  modules would be a local refactor with no data migration.
* Bad, because `pirs-core::export` now contains both export and import envelope
  handling, so the module name is less exact until the code grows enough to
  justify a separate JSON-PIR module.
* Bad, because redaction is configuration-driven and cannot prove that every
  future sensitive custom field is covered.

### Confirmation

Compliance is confirmed by the following checks:

* `pirs export json` without `--redact` continues to produce the existing JSON
  shape.
* `pirs export json --redact` masks configured regex matches and sensitive
  fields before stdout receives JSON.
* `pirs import json <FILE>` and `pirs import json -` both parse single and bulk
  JSON-PIR v1 documents.
* `pirs import json --dry-run` reports NEW/SKIP/OVERWRITE actions without
  writing files.
* Existing-number imports skip by default and overwrite only when
  `--overwrite` is supplied.
* Core redaction and import parsing helpers have direct tests or CLI
  integration tests that exercise their behavior.

## Pros and Cons of the Options

### Put JSON-PIR policy in `pirs-core`

This option keeps command modules as side-effect boundaries while moving
redaction and JSON-PIR document behavior into reusable library functions.

* Good, because it matches the existing split between `pirs` as CLI shell and
  `pirs-core` as reusable data/model logic.
* Good, because tests can exercise redaction and JSON parsing without spawning
  the CLI.
* Good, because it avoids future duplication if import/export appears in MCP
  tools.
* Neutral, because it adds several public helper types to `pirs-core`.
* Bad, because the existing `export.rs` module becomes responsible for import
  envelope parsing too, unless later renamed or split.

### Keep most behavior in CLI command modules

This option would implement parsing, redaction, dry-run planning, and overwrite
policy mostly in `crates/pirs/src/commands/export.rs` and a new
`commands/import.rs`.

* Good, because it keeps the initial implementation close to the subcommands
  that expose it.
* Good, because it minimizes changes to the library API.
* Neutral, because command integration tests can still cover the feature.
* Bad, because redaction and import policy would be harder to reuse outside the
  CLI.
* Bad, because testing pure behavior would require more subprocess or
  filesystem setup.

## More Information

The implementation should avoid printing unredacted JSON in error messages.
Large streaming imports are out of scope for the first implementation; the
initial importer may read a full JSON-PIR document into memory, which is
acceptable for normal PIR repository sizes. Revisit this ADR if JSON-PIR import
becomes a high-volume migration path or if MCP tools need a separate module
boundary for JSON-PIR operations.
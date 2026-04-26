---
number: 5
title: Single mcp.rs module rather than mcp directory
status: accepted
date: 2026-04-26
---

# Single mcp.rs module rather than mcp directory

## Context and Problem Statement

We need to decide how to organize the MCP-related Rust code while the
implementation is still small and cohesive. The immediate choice is whether
to keep all of the functionality in a single `crates/pirs/src/mcp.rs`
module or to create an `mcp/` directory with multiple files such as
`mod.rs`, `state.rs`, `read_tools.rs`, and `write_tools.rs`.

A directory-based layout is useful once the code grows, but it also
introduces extra files, indirection, and maintenance overhead. The question
for this ADR is whether the current scope justifies that additional
structure now.

## Decision Drivers

* Keep the codebase easy to navigate for a small MCP implementation
  (~700 LOC, 13 tools).
* Minimize file and module boilerplate while the logic remains closely
  related and read mostly top-to-bottom.
* Preserve the ability to refactor into a directory later if the module
  grows.
* Prefer straightforward organization over speculative decomposition.

## Considered Options

* Keep a single `mcp.rs` module.
* Split the code into an `mcp/` directory now.
* Use a single `mcp.rs` module today and refactor later if growth justifies
  it.

## Decision Outcome

Chosen option: **"Keep a single `mcp.rs` module"**, because the current MCP
code is small enough to remain understandable in one file, and introducing
a directory structure now would add ceremony without practical benefit. The
13 tool handlers are short and follow the same pattern, so reading them in
sequence is easier than chasing them across multiple files.

### Consequences

* Good, because related MCP logic remains in one place and is easy to
  discover.
* Good, because the project avoids premature modularization and extra file
  churn.
* Good, because refactoring from one module into a directory remains
  straightforward if the implementation expands.
* Bad, because a larger future `mcp.rs` file may eventually become harder
  to scan.
* Bad, because a single-file layout provides less explicit separation
  between read tools, write tools, and transport wiring.

### Confirmation

Compliance is confirmed by repository structure review. As long as
`crates/pirs/src/mcp.rs` stays under roughly 1000 lines and changes review
comfortably within a single module, the ADR is being followed. If `mcp.rs`
starts to mix clearly separate responsibilities (e.g., a dedicated
authentication layer, multiple transports beyond stdio/HTTP, or a
significantly larger tool surface), this decision should be revisited and
the code can be split into an `mcp/` directory.

## Pros and Cons of the Options

### Keep a single `mcp.rs` module

* Good, because it is the simplest structure for a small, cohesive
  implementation.
* Good, because contributors only need to look in one place to understand
  the MCP code.
* Neutral, because it does not prevent a later refactor into multiple
  files.
* Bad, because file size will grow over time as new tools are added.

### Split the code into an `mcp/` directory now

* Good, because responsibilities can be separated earlier into dedicated
  files (read vs write, schemas, helpers).
* Neutral, because Rust supports either organization style cleanly.
* Bad, because it adds boilerplate and indirection before the complexity
  requires it.
* Bad, because it spreads ~700 LOC of closely related logic across multiple
  files unnecessarily.

### Use a single `mcp.rs` module now and refactor later if needed

* Good, because it keeps today's structure simple while acknowledging
  growth.
* Neutral, because it is operationally close to the chosen option, just
  with an explicit review trigger.
* Bad, because without clear thresholds, the refactor point may remain
  subjective.

## More Information

Revisit if `mcp.rs` grows to the point where separate concerns such as
protocol types, transport handling, authentication, and higher-level
orchestration no longer fit comfortably in one module. Until then, prefer
the simpler layout.

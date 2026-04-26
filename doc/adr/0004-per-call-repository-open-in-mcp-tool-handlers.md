---
number: 4
title: Per-call Repository::open in MCP tool handlers
status: accepted
date: 2026-04-26
---

# Per-call Repository::open in MCP tool handlers

## Context and Problem Statement

MCP tool handlers may be invoked independently and concurrently, with no
guarantees about shared in-memory state between calls. We need a predictable
way for each handler to access the on-disk PIR repository while avoiding
stale state, hidden coupling between tool invocations, and long-lived
repository handles that are difficult to reason about — especially since
the on-disk layout can be modified by the user (or another `pirs` process)
in between calls.

The question is whether handlers should reuse a single shared repository
object held in `PirState` or call `Repository::open(&state.root)` for every
tool call.

## Decision Drivers

* REQ-MCP-002: tool handlers must observe the current on-disk state, even
  if the user edits files outside the server.
* Tool handlers must behave correctly under concurrent or interleaved
  calls.
* Repository access should not depend on mutable shared state across
  requests.
* Resource lifecycle should be explicit and bounded to a single tool
  invocation.
* Implementation should be simple to read, test, and review.

## Considered Options

* Open the repository per tool call inside each MCP handler.
* Reuse a shared repository instance for the lifetime of the MCP server
  process.
* Cache repository instances and reuse them opportunistically across calls.

## Decision Outcome

Chosen option: **"Open the repository per tool call inside each MCP
handler"**, because it provides the clearest lifecycle boundary, guarantees
that each call sees the current on-disk state, and is the safest default
for correctness in a request-oriented integration. `Repository::open` is
cheap (it parses `pirs.toml` and lists a directory); profiling did not show
it as a hotspot.

### Consequences

* Good, because each tool invocation gets a fresh repository handle with a
  well-defined lifetime.
* Good, because concurrent calls are isolated from one another and cannot
  interfere through shared in-memory caches.
* Good, because external file edits between calls are picked up
  automatically — important when users mix CLI and MCP usage.
* Good, because failure handling is simpler when acquisition and cleanup
  happen within the same handler.
* Bad, because opening the repository on every call adds modest overhead
  compared with reusing a long-lived instance.
* Bad, because handlers must consistently call `open_repo(&st)` rather than
  relying on process-level initialization. This is mitigated by the
  `open_repo` helper.

### Confirmation

Compliance is confirmed by code review of `crates/pirs/src/mcp.rs`: every
tool handler calls `open_repo(&st)` (which delegates to
`Repository::open`) at the top of its body. The
`mcp_full_lifecycle_resolved_with_actions_and_whys` integration test makes
nine sequential tool calls in one server process and observes that each
call sees the file written by the previous call.

## Pros and Cons of the Options

### Open the repository per tool call inside each MCP handler

* Good, because repository state is fresh for each invocation.
* Good, because the lifecycle is explicit and easy to reason about.
* Neutral, because the pattern duplicates two lines of setup across
  handlers (wrapped in `open_repo` to keep this trivial).
* Bad, because repeated open operations cost more than reusing an open
  instance.

### Reuse a shared repository instance for the lifetime of the MCP server process

* Good, because it may reduce per-call overhead.
* Bad, because shared mutable state increases the risk of stale data,
  lifecycle bugs, and cross-request coupling.
* Bad, because external file edits between calls would not be observed
  without explicit invalidation logic.

### Cache repository instances and reuse them opportunistically across calls

* Good, because it could balance some performance benefits with fewer
  opens.
* Bad, because correctness depends on cache invalidation rules that have
  not been needed.
* Bad, because it introduces more complexity than either purely per-call
  or purely shared approaches.

## More Information

This decision prefers correctness, isolation, and operational simplicity
over speculative optimization. If profiling later shows that per-call
repository opening is a meaningful bottleneck, the team can revisit this
ADR with concrete measurements and a clear proposal for safe reuse or
caching semantics.

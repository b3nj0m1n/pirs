---
number: 6
title: Server-level --agent flag as default actor for write tools
status: accepted
date: 2026-04-26
---

# Server-level --agent flag as default actor for write tools

## Context and Problem Statement

Several MCP write tools (`create_pir`, `append_timeline_event`, `add_why`,
`add_action`, `update_action`, `update_status`, `link_evidence`) need an
actor name to record who performed the change. In the common deployment
case there is exactly one agent talking to the server (e.g., one Claude
client), and forcing every tool call to repeat the same `agent` argument is
verbose, error-prone, and clutters the schemas the agent sees.

We need a way to make the common single-agent case ergonomic without
breaking the multi-agent case where different callers act under different
identities through one server process.

## Decision Drivers

* Reduce per-call boilerplate for the dominant single-agent deployment.
* Keep schemas understandable: agents can see the `agent` argument is
  optional rather than always required.
* Continue to support multi-agent setups where each call carries its own
  identity.
* Make the failure mode (no actor available at all) clear and early.

## Considered Options

* Server-level `--agent` flag that supplies the default actor; per-call
  `agent` argument overrides it.
* Per-call `agent` argument required on every write tool, with no
  server-level default.
* Server-level `--agent` flag only, with no per-call override.

## Decision Outcome

Chosen option: **"Server-level `--agent` flag with optional per-call
`agent` argument that overrides"**. `PirState` carries
`agent: Option<String>`; write handlers fall back to it when the call omits
one and return an error if both are absent.

### Consequences

* Good, because the common case (one agent per server) needs no per-call
  boilerplate or repeated arguments.
* Good, because multi-agent deployments still work via the per-call
  argument.
* Good, because the failure mode is explicit: a write tool called with no
  per-call `agent` and no server default returns an error rather than
  silently writing an "unknown" actor.
* Neutral, because operators must remember to pass `--agent` for the
  most ergonomic experience.

### Confirmation

Confirmed by code review of `crates/pirs/src/mcp.rs`: each write handler
resolves the actor via `resolve_actor(&st, input.agent.as_deref())`,
which prefers the per-call argument and falls back to `st.agent`. The
`mcp_full_lifecycle_resolved_with_actions_and_whys` test passes a
per-call `agent` on every call to assert that override path works; manual
verification of the `--agent` fallback path was performed against a
scratch repo at `/tmp/mcp-smoke`.

## Pros and Cons of the Options

### Server-level `--agent` flag with per-call override (chosen)

* Good, because it makes the common case ergonomic.
* Good, because it preserves correctness for multi-agent deployments.
* Good, because it produces a clear error when no actor is available.
* Bad, because the resolution rule (per-call > server default > error) must
  be documented for users.

### Per-call `agent` argument required on every write tool

* Good, because the call site is always self-describing.
* Bad, because it forces the same string into every tool call in the
  single-agent case.
* Bad, because it makes the schemas noisier and increases the risk of
  agents forgetting the argument.

### Server-level `--agent` only, with no per-call override

* Good, because it is the simplest shape.
* Bad, because it cannot represent multi-agent setups.
* Bad, because changing actor mid-session would require restarting the
  server.

## More Information

Revisit if multi-agent deployments become the dominant case (e.g., one
shared MCP server fronting several agents), in which case requiring the
per-call argument may be a better default than allowing it to be omitted.

# MCP Security Expectations

This chapter implements **REQ-DOC-009**: the threat model and operational
guidance for running `pirs mcp serve`. It is required reading before
exposing the MCP server beyond a single local agent.

## Default posture: stdio only

`pirs mcp serve` defaults to stdio transport. The HTTP transport is gated
behind the `http-transport` Cargo feature (see
[ADR-0003](https://github.com/joshrotenberg/pirs/blob/main/doc/adr/0003-stdio-transport-default-http-gated-behind-cargo-feature.md)).
Rationale:

- Stdio is point-to-point: a parent process spawns the server and owns the
  pipe. There is no listening socket, no cross-origin attack surface, and no
  authentication problem to solve.
- HTTP introduces a new trust boundary, requires authentication and
  authorisation, and risks exposing PIR contents over the network.

If you need HTTP, build with `cargo build --features http-transport` and
treat the resulting binary as if it were any other HTTP service: terminate
TLS, add authentication in front, and bind only to trusted interfaces.

## What the server can do

Every CLI write operation is mirrored as an MCP tool. An attacker with a tool
channel can:

- Create, modify, and (with `pir_status_set`) advance arbitrary PIRs.
- Read all on-disk PIR contents, including `Confidential` ones.
- Trigger `pir_export_json` and read raw JSON.
- Trigger `pir_import_json` to inject crafted PIRs.

The server **cannot**:

- Execute arbitrary shell commands.
- Read files outside the configured PIR directory.
- Modify configuration (`pirs.toml`); configuration is read-only at server
  start.

## Per-call repository open

The server does not hold a long-lived `Repository` handle. Each tool
invocation re-opens the repository from the configured directory (see
[ADR-0004](https://github.com/joshrotenberg/pirs/blob/main/doc/adr/0004-per-call-repository-open-in-mcp-tool-handlers.md)).
This means:

- Configuration changes on disk take effect immediately on the next call.
- Concurrent CLI use against the same repository is safe.
- A buggy tool handler cannot leak file handles across calls.

## Actor identity

Every write tool requires an actor. There are two ways to provide it:

1. Per call: pass `actor` in the tool arguments.
2. Server-level default: start the server with `--agent "<name>"` to install
   a default actor (see
   [ADR-0006](https://github.com/joshrotenberg/pirs/blob/main/doc/adr/0006-server-level-agent-flag-as-default-actor-for-write-tools.md)).

If neither is present, the tool returns a structured error rather than
guessing or attributing to "system".

## Redaction

`pirs export json --redact` and the `pir_export_json` MCP tool with
`redact: true` apply the redaction patterns from `[privacy]` in `pirs.toml`.
Patterns are full Rust regular expressions; the matched substring is
replaced with `[REDACTED]`.

Redaction is **best-effort**:

- Patterns are only applied to fields named in `sensitive_fields` plus the
  `problem_statement`, `impact`, `summary`, timeline descriptions, and 5-Whys
  answers.
- Unknown secret formats (a credential the patterns don't match) will not be
  redacted. Treat redacted exports as low-sensitivity, not zero-sensitivity.
- Redaction is applied at *export* time, not at write time. The on-disk
  Markdown remains the source of truth and may contain unredacted material.

If you need stronger guarantees, scrub the input before passing it to
`pirs new` / the MCP tools, and consider classifying the PIR as
`Confidential` so that future exports are gated.

## Operational checklist

Before letting an LLM agent talk to a long-running MCP server:

1. Confirm the binary was built without `http-transport`, or that HTTP is
   bound to localhost behind authentication.
2. Set `--agent "<canonical name>"` so writes are attributed.
3. Audit `pirs.toml`'s `[privacy]` block; add any project-specific token
   formats.
4. Ensure the PIR directory is in version control so unintended writes are
   visible in the next diff.
5. Decide your policy on `Confidential` and `Restricted` PIRs — agents
   should typically not be permitted to read or export them. Today this is
   enforced socially; finer-grained ACLs are tracked as future work in the
   spec.

---
number: 3
title: Stdio transport default; HTTP gated behind cargo feature
status: accepted
date: 2026-04-26
---

# Stdio transport default; HTTP gated behind cargo feature

## Context and Problem Statement

The MCP server supports more than one transport for communicating with
clients. We need a sensible default that works in the widest range of local
and tool-driven environments without forcing all consumers to compile in
networking support. We also want HTTP transport to remain available for
deployments that need it, but only when explicitly enabled.

## Decision Drivers

* Keep the default build small and free of HTTP/TLS dependency surface.
* Prefer the transport that works reliably for local process-based
  integrations (Claude Desktop, Continue, editor plugins).
* Avoid forcing HTTP-only configuration on consumers that do not need it.
* Preserve the ability to enable HTTP transport for deployments that do.

## Considered Options

* Make stdio the default transport and gate HTTP behind a Cargo feature.
* Make HTTP the default transport and gate stdio behind a Cargo feature.
* Ship both transports unconditionally with no feature gating.

## Decision Outcome

Chosen option: **"Make stdio the default transport and gate HTTP behind a
Cargo feature"**, because it provides the lowest-friction default for local
integrations, keeps the default dependency graph small, and still allows
HTTP transport to be enabled explicitly when a deployment requires it.

### Consequences

* Good, because the default build path stays focused on local/process-based
  use, which matches every existing MCP client today.
* Good, because consumers that only need stdio do not pay the build,
  dependency, or runtime cost of HTTP support.
* Good, because HTTP remains supported for environments that need a network
  transport — `cargo build --features http` is enough.
* Bad, because users who expect HTTP out of the box must discover and enable
  the appropriate Cargo feature.
* Bad, because documentation and packaging must clearly describe which
  feature enables HTTP transport.

### Confirmation

Compliance is confirmed by reviewing `crates/pirs/Cargo.toml` (the `http`
feature wires `tower-mcp/http`) and `crates/pirs/src/mcp.rs` (the
`HttpTransport` path is behind `#[cfg(feature = "http")]`). `cargo build -p
pirs` produces a binary with stdio only; `cargo build -p pirs --features
http` produces one that can also serve HTTP. Both build configurations are
exercised by CI/local testing.

## Pros and Cons of the Options

### Make stdio the default transport and gate HTTP behind a Cargo feature

* Good, because it aligns the default with local tool/process invocation
  patterns used by every current MCP client.
* Good, because it reduces default dependency surface area.
* Neutral, because HTTP is still available when explicitly requested.
* Bad, because HTTP users must take an extra configuration step.

### Make HTTP the default transport and gate stdio behind a Cargo feature

* Good, because networked deployments work without feature selection.
* Neutral, because stdio could still be supported as an optional mode.
* Bad, because it makes the default build heavier for consumers that only
  need local stdio communication.
* Bad, because it treats a deployment-specific transport as the baseline
  even though no current MCP client uses it by default.

### Ship both transports by default with no feature gating

* Good, because all supported transports are immediately available.
* Neutral, because users do not need to learn feature flags.
* Bad, because it increases the default dependency and build surface for
  every consumer regardless of need.
* Bad, because it weakens the separation between common and optional
  functionality.

## More Information

This decision should be revisited if the primary deployment model changes
from local/process-based integrations to predominantly networked
deployments, or if maintaining HTTP as an optional feature introduces
unacceptable complexity. Related implementation and release documentation
should clearly call out that stdio is the default and HTTP requires the
explicit `http` Cargo feature.

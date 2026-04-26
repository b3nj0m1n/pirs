---
number: 2
title: Use tower-mcp 0.10 for MCP server implementation
status: accepted
date: 2026-04-26
---

# Use tower-mcp 0.10 for MCP server implementation

## Context and Problem Statement

REQ-MCP-001..006 require an MCP server exposing PIR read and write tools.
Several Rust MCP crates exist (`rmcp`, `mcp-rs`, `tower-mcp`, hand-rolled
JSON-RPC). We need a stable choice that minimises bespoke transport code.

## Decision Drivers

* Spec stated preference for tower-mcp (REQ-MCP-001 explicit reference).
* Author overlap with existing `adrs` / `pirs` tooling lineage — same idioms.
* Need both stdio (default) and optional HTTP transports.
* axum-style extractor pattern keeps tool handlers declarative.

## Considered Options

* tower-mcp 0.10
* rmcp (official-ish) crate
* Hand-rolled JSON-RPC over stdio

## Decision Outcome

Chosen: **tower-mcp 0.10** with `default-features = false` and an opt-in
`http` feature flag wired through to `tower-mcp/http`. The router/builder
API maps cleanly onto our 13 tools; `StdioTransport` and `HttpTransport`
satisfy REQ-MCP-001 without us writing transport plumbing.

### Consequences

* Good — declarative `ToolBuilder::extractor_handler` keeps tool code small
  and gets schema generation from `schemars` for free.
* Good — dependency-graph alignment with the project author's other tools.
* Bad — couples us to `schemars` 1.x (tower-mcp 0.10 transitive); a major
  bump there will require coordinated updates (see PIR-0001).
* Bad — fewer eyes than rmcp; bug fixes may take longer.

### Confirmation

`cargo build -p pirs` and `cargo build -p pirs --features http` both clean;
`crates/pirs/tests/mcp.rs` exercises stdio JSON-RPC end-to-end.

# Introduction

`pirs` is a CLI and library for managing **Post-Incident Reviews (PIRs)**. A
PIR is a structured record of an incident — a failing test, a broken build, a
bad migration, a production outage, a security event, or a process failure —
together with its timeline, 5-Whys analysis, and the follow-up actions needed
to prevent recurrence.

This documentation is the task-oriented user guide. It complements:

- The project [README](https://github.com/joshrotenberg/pirs#readme), which is
  a quick-start.
- The [requirements specification](https://github.com/joshrotenberg/pirs/blob/main/spec/pirs_requirements_spec.md),
  which is the build target and validation checklist.
- The [Architecture Decision Records](https://github.com/joshrotenberg/pirs/tree/main/doc/adr),
  which capture *why* the implementation looks the way it does.

## What pirs is for

| Audience | Primary use |
|---|---|
| Solo developers | Capture a failing test or broken build in seconds; review later. |
| Teams | Maintain a versioned record of incidents alongside the code. |
| LLM agents | Log incidents and decisions through CLI or the MCP server while working. |

## Design principles

1. **Cheap to record.** A failing command becomes a PIR with one wrapped
   invocation: `pirs run --on-fail create -- cargo test`.
2. **Markdown + YAML on disk.** PIRs live under `doc/pir/` so they review
   naturally in pull requests.
3. **Blameless by default.** The tool warns on blame-oriented language and
   records actors as humans, agents, teams, or systems.
4. **Action-oriented.** Follow-up actions are first-class entities with
   owners, due dates, statuses, and evidence.
5. **Agent-friendly.** Every write subcommand has a non-interactive form, and
   the same operations are exposed as MCP tools.

## How this book is organised

- **[Getting Started](./getting-started.md)** walks through `init`, creating a
  PIR, adding timeline entries, recording a 5-Whys chain, and resolving.
- **[PIR File Format](./file-format.md)** documents the YAML frontmatter
  fields and the Markdown section conventions parsed by `pirs`.
- **[Agent Workflow Patterns](./agent-workflows.md)** is the LLM-facing guide:
  when to log a PIR, when to record an ADR, how to wrap commands, and how to
  use the MCP write tools.
- **[Severity and Status Taxonomy](./severity-and-status.md)** is the
  reference for the controlled vocabularies attached to each PIR.
- **[MCP Security Expectations](./mcp-security.md)** covers the threat model
  for the MCP server, including the HTTP-transport feature flag.

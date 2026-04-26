# Agent Workflow Patterns

This chapter implements **REQ-DOC-007**: a guide for LLM agents (and the
humans supervising them) on when and how to use `pirs`. It assumes you have
read [Getting Started](./getting-started.md) and the
[PIR File Format](./file-format.md).

## Decision matrix: ADR or PIR?

| Symptom | Record as |
|---|---|
| You made an *architectural choice* (library, transport, schema, layering). | **ADR** in a sibling `adrs` repo. |
| Something *broke*: failing test, broken build, bad deploy, regression. | **PIR**, type `development` or `production`. |
| Sensitive data leaked, secret committed, suspicious access. | **PIR**, type `security`, confidentiality at least `Confidential`. |
| Process or runbook step was skipped, stale, or ambiguous. | **PIR**, type `process`. |
| You investigated and found *no* defect. | **No PIR**. Note in commit or PR body. |

If a single event needs both an architecture course-correction and an
incident record, file both and link the PIR to the ADR via `links` (see
`LinkKind::ADR`).

## Pattern 1: wrap a failing command

Use when an automated step (test, lint, migration, deploy) might fail and you
want the failure recorded automatically.

```sh
pirs run --on-fail create \
  --agent "GitHub Copilot" \
  -- cargo test --workspace
```

`--on-fail` accepts `create` (default) or `none`. Pair with `--pir N` to
append a timeline event to an existing PIR instead of creating a new one,
and with `--always-log` to record an entry even on success. The wrapped
command's exit code is preserved.

## Pattern 2: log an incident discovered mid-task

When an agent notices something is broken in the middle of unrelated work:

1. Pause the original task.
2. `pirs new "<short title>" --type <type> --severity <level> --agent "<agent name>" --no-edit --problem "<one paragraph>"`.
3. `pirs timeline add <N> --actor "<agent>" --type detected --message "..."`.
4. Decide whether to fix now (continue with the same PIR open) or hand off
   (set status to `Investigating` and surface to a human reviewer).

## Pattern 3: complete a 5-Whys chain

Each `pirs why add` call appends a question/answer pair. A typical chain is
3–5 entries; use `--as-root-cause` on the entry whose answer is the actionable
root cause.

```sh
pirs why add 12 --question "why did the payment hang?" --answer "DB connection pool exhausted"
pirs why add 12 --question "why was the pool exhausted?" --answer "long-running analytics query"
pirs why add 12 --question "why did analytics run on the live pool?" \
                --answer "no read-replica configured" --as-root-cause
```

`--as-root-cause` writes to the PIR-level `root_cause` field, which is a
prerequisite for the `Reviewed` status transition.

## Pattern 4: assign and discharge actions

```sh
pirs action add 12 \
  --description "Move analytics queries to read-replica" \
  --owner "platform-team" --owner-type team \
  --due 2026-05-15

# When complete:
pirs action close 12 ACT-001 --evidence "https://github.com/example/repo/pull/42"
```

`--owner-type` should reflect reality. If an agent will execute the action,
use `agent`; if a team or person owns it, use `team` or `human`. Do **not**
record agent commit hashes or timestamps as evidence — link to the PR or
deployment that verifies the change.

## Pattern 5: use the MCP server

For tool-using agents, `pirs mcp serve` exposes the same operations over the
Model Context Protocol. By default it speaks stdio; HTTP is gated behind the
`http-transport` cargo feature for the reasons in
[MCP Security Expectations](./mcp-security.md).

```sh
pirs mcp serve                       # stdio, agent identity required per call
pirs mcp serve --agent "Copilot"     # set a default actor for write tools
```

Available tools include `create_pir`, `list_pirs`, `get_pir`, `search_pirs`,
`append_timeline_event`, `add_why`, `add_action`, `update_action`,
`update_status`, `link_evidence`, `get_open_actions`, `get_repository_info`,
and `validate_pir`. Each tool returns a JSON envelope with `status`, the
created or updated PIR/action identifiers, and any warnings.

## Best practices for agents

1. **Prefer non-interactive flags.** Pass `--no-edit`, `--problem`,
   `--severity`, and `--agent` instead of opening `$EDITOR`.
2. **Always identify yourself.** Use `--agent "<canonical agent name>"` so
   timelines stay traceable.
3. **One PIR per incident.** If multiple symptoms share a root cause, file
   one PIR and link the others; do not duplicate.
4. **Don't fabricate timestamps.** Let `pirs` set them via `--now` or omit
   the field entirely if unknown.
5. **Run `pirs doctor` before requesting review.** It enforces the
   `Reviewed`-gate prerequisites.
6. **Never paste secrets into a PIR.** The redaction pipeline only catches
   configured patterns; treat fields like log excerpts as semi-public.

# pirs

A CLI and library for **Post-Incident Reviews (PIRs)**, modelled after
[`adrs`](https://github.com/joshrotenberg/adrs). Designed for both humans and LLM agents: makes it cheap to
record a failing test, broken build, bad migration, or production outage, and
turns those records into structured learning artefacts with timelines, 5 Whys
analysis, and tracked follow-up actions.

PIRs are stored as Markdown with YAML frontmatter under `doc/pir/` by default,
so they live naturally next to your code in git.

> Status: initial bootstrap. See [spec/pirs_requirements_spec.md](https://github.com/b3j0m1n/spec/pirs_requirements_spec.md)
> for the full requirements specification.

## Quick start

```bash
# initialize a repository
pirs init

# create an agent-driven development PIR (non-interactive)
pirs new "Failing cargo test after parser change" \
  --type development \
  --severity medium \
  --agent "GitHub Copilot" \
  --problem "cargo test failed after parser metadata update" \
  --no-edit

# wrap a command and auto-create a PIR if it fails (preserves exit code)
pirs run --on-fail create --agent "GitHub Copilot" -- cargo test

# work the PIR
pirs timeline add 1 --actor "GitHub Copilot" --type investigated --message "found stale cache"
pirs why add 1 --question "why did tests fail?" --answer "stale cache" --as-root-cause
pirs action add 1 --description "Add regression test" --owner "GitHub Copilot" --owner-type agent --due 2026-12-31

# resolve and review
pirs status 1 resolved --now
pirs status 1 reviewed   # gated: requires problem_statement, timeline, 5 whys, actions, resolved_at

# inspect
pirs list -l
pirs show 1
pirs actions --owner "GitHub Copilot"
pirs doctor
pirs export json > pirs.json
pirs export json --redact > pirs-redacted.json
pirs import json pirs.json --dry-run
pirs import json pirs.json --overwrite

# expose the same operations to LLM agents over the Model Context Protocol
pirs mcp serve                                  # stdio (REQ-MCP-001)
pirs mcp serve --agent "GitHub Copilot"        # default actor for write tools
# build with HTTP transport enabled (REQ-MCP-006: warns on non-loopback bind):
#   cargo build --features http
#   pirs mcp serve --http 127.0.0.1:7878
```

## Workspace

```text
PIRS/
├── Cargo.toml                   # workspace manifest
├── crates/
│   ├── pirs/                    # CLI binary
│   └── pirs-core/               # core library: types, parser, repository, lint, templates, export
└── schema/json-pir/v1.json      # JSON-PIR schema
```

## Implemented (this bootstrap)

- `init`, `new`, `list`, `show`, `search`, `status`, `why add`, `action add|close`,
  `actions`, `timeline add`, `people add`, `link`, `doctor` (+ `--review-gate`),
  `export json`, `config`, `template list|show`, `run --on-fail create|append|none`.
- `mcp serve` — MCP server exposing 6 read tools (`list_pirs`, `get_pir`,
  `search_pirs`, `get_open_actions`, `get_repository_info`, `validate_pir`)
  and 7 write tools (`create_pir`, `append_timeline_event`, `update_status`,
  `add_why`, `add_action`, `update_action`, `link_evidence`) over stdio, plus
  optional HTTP transport behind the `http` cargo feature
  (REQ-MCP-001..006).
- Built-in templates: `development`, `production`, `security`, `process`, `minimal`.
- YAML frontmatter parser, atomic file writes, ISO-8601 duration derivation.
- JSON-PIR v1 schema, bulk/single export, redacted export (`--redact`), and
  import from files or stdin with dry-run and overwrite handling.
- 26 tests covering acceptance criteria AC-001/002/003/005/006/007/009/010/011,
  REQ-TIME-003 and the MCP server lifecycle.

## Not yet implemented

- Reports: `generate report`, `generate actions`, `metrics`, blameless
  language audit (`doctor --language`).
- File locking around `next_number()` (basic atomic write only).
- Shell completions, mdBook documentation, fixture corpus.

## License

Dual-licensed under MIT or Apache-2.0, matching the parent project.

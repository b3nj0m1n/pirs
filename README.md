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
- Built-in templates: `development`, `production`, `security`, `process`, `minimal`.
- YAML frontmatter parser, atomic file writes, ISO-8601 duration derivation.
- JSON-PIR v1 schema and bulk/single export.
- 15 tests covering acceptance criteria AC-001/002/003/005/006/007/009/010/011 and
  REQ-TIME-003.

## Not yet implemented

- MCP server (read + write tools).
- `import json` / redacted export (`--redact`).
- Reports: `generate report`, `generate actions`, `metrics`, blameless
  language audit (`doctor --language`).
- File locking around `next_number()` (basic atomic write only).
- Shell completions, mdBook documentation, fixture corpus.

## License

Dual-licensed under MIT or Apache-2.0, matching the parent project.

# Getting Started

This chapter implements **REQ-DOC-005**: a tutorial that takes a new user
from an empty repository through resolving and reviewing a PIR.

## Install

`pirs` is a Rust workspace. Until released crates are published, build from
source:

```sh
git clone https://github.com/joshrotenberg/pirs
cd pirs
cargo install --path crates/pirs
```

Confirm the install:

```sh
pirs --version
```

## Initialise a repository

From the root of any project:

```sh
pirs init
```

This creates `doc/pir/` (the default repository directory) and a top-level
`pirs.toml`. The directory is configurable through `pirs.toml`, the legacy
`.pir-dir` marker, or the `$PIR_DIRECTORY` environment variable. Run
`pirs config` to inspect the resolved configuration.

## Create your first PIR

Non-interactive form (recommended for both humans-in-a-hurry and LLM agents):

```sh
pirs new "Failing cargo test after parser change" \
  --type development \
  --severity medium \
  --agent "GitHub Copilot" \
  --problem "cargo test failed after parser metadata update" \
  --no-edit
```

The PIR is written to `doc/pir/0001-failing-cargo-test-after-parser-change.md`.
Inspect it with `pirs show 1` or `pirs list -l`.

## Wrap a failing command

`pirs run` captures any non-zero exit and converts it into a development PIR
without losing the underlying exit code:

```sh
pirs run --on-fail create --agent "GitHub Copilot" -- cargo test
```

If `cargo test` fails, a PIR is created (or appended to, with `--pir`), the
command output is captured (after redaction; see
[MCP Security Expectations](./mcp-security.md)), and the original exit code
is propagated.

## Add timeline events

Every meaningful step earns a timeline entry. Actors should always be named;
use `--actor "GitHub Copilot"` for agents.

```sh
pirs timeline add 1 \
  --actor "GitHub Copilot" \
  --type investigated \
  --message "found stale cache"
```

## Record the 5 Whys

Append answers in order; mark exactly one as the root cause:

```sh
pirs why add 1 --question "why did tests fail?"        --answer "stale cache"
pirs why add 1 --question "why was the cache stale?"   --answer "build script skipped invalidation" --as-root-cause
```

`--as-root-cause` populates the PIR-level `root_cause` field; the doctor
command will block promotion to `Reviewed` if no root cause is recorded.

## Track follow-up actions

```sh
pirs action add 1 \
  --description "Add regression test for cache invalidation" \
  --owner "GitHub Copilot" --owner-type agent \
  --due 2026-12-31
```

List the cross-PIR action register at any time:

```sh
pirs actions --owner "GitHub Copilot"
```

## Resolve and review

```sh
pirs status 1 resolved --now
pirs status 1 reviewed   # gated by `pirs doctor --review-gate 1`
```

The `Reviewed` transition is gated: a PIR must have a problem statement,
timeline events, a 5-Whys chain with a root cause, at least one action item,
and `resolved_at` set. Run `pirs doctor` to see what's missing.

## Inspect the repository

```sh
pirs list -l           # detailed list
pirs show 1            # render a single PIR
pirs metrics           # incident counts, TTD/TTR, recurring tags
pirs metrics --json    # machine-readable form
pirs doctor            # health check
pirs doctor --language # blame-oriented phrasing warnings
```

## Export and import

```sh
pirs export json > pirs.json
pirs export json --redact > pirs-redacted.json
pirs import json pirs.json --dry-run
```

See the [PIR File Format](./file-format.md) chapter for the JSON-PIR v1
schema and the on-disk Markdown layout, and
[MCP Security Expectations](./mcp-security.md) for redaction policy.

---
review: feat-completions-mdbook-fixtures
date: 2026-04-26
reviewers: code-reviewer (Explore subagent), security-reviewer (Explore subagent)
branch: feat/completions-mdbook-fixtures
---

# Pre-merge review: shell completions, mdBook, fixture corpus

Two parallel reviews were dispatched per the SDLC-orchestrator skill's
mandatory pre-merge gate. This report consolidates findings, triage, and the
fixes that were landed before merge.

## Code review summary

| Severity | Count | Status |
|---|---|---|
| Critical | 1 | Dismissed (false positive) |
| High | 1 | **Fixed** |
| Medium | 1 | Dismissed |
| Low | 0 | — |
| Nit | 0 | — |

### Critical — dismissed (false positive)

> `commands/completions.rs:28` — `clap_complete::generate()` return value
> ignored, silencing I/O errors.

The signature of `clap_complete::generate` is
`fn generate<G, S>(gen: G, cmd: &mut Command, bin_name: S, buf: &mut dyn Write)`
returning `()`. There is no `Result` to propagate — internal write failures
panic inside the generator, which is consistent with `println!` and
acceptable for a one-shot CLI subcommand. No change required.

### High — FIXED

> `book/src/agent-workflows.md:30` — Pattern 1 example references
> non-existent `--pir-type` and `--severity` flags on `pirs run`.

Confirmed against `crates/pirs/src/main.rs` lines 240–258: `Run` only
exposes `--on-fail`, `--pir`, `--agent`, `--always-log`. The example was
rewritten to use the real flags and clarify `--on-fail` accepts only
`create` or `none`, with `--pir N` for append behaviour.

### Medium — dismissed

> `book/book.toml:7` — `edit-url-template` references `main` branch.

This is the desired behaviour: the book is published from `main`, and the
template should resolve to the merged location. No change.

## Security review summary

| Severity | Count | Status |
|---|---|---|
| Critical | 0 | — |
| High | 1 | Accepted risk |
| Medium | 1 | Dismissed |
| Low | 2 | 1 fixed / 1 accepted |
| Informational | 3 | Documented |

### High — accepted risk

> `commands/completions.rs:33-37` — Symlink TOCTOU race in `--out-dir`.

`fs::create_dir_all()` follows symlinks. An attacker who can pre-create a
symlink at the user-chosen `--out-dir` could redirect completion writes
elsewhere. Triage:

- `pirs completions` is a developer-local CLI invoked by the user; the
  user controls `--out-dir`. There is no privilege boundary being crossed.
- The standard installation pattern (`pirs completions zsh
  --out-dir ~/.zsh/completions`) writes to a directory owned by the same
  user.
- Any mitigation (canonicalisation, refusing symlinked targets) would
  break legitimate use cases such as `--out-dir` pointing at a symlink to
  the user's dotfiles repo.

We accept the risk and document it: users running as root or pointing
`--out-dir` at a directory they do not control should reconsider.

### Medium — dismissed

> `commands/completions.rs:20-22` — Unvalidated binary name in completion
> output.

`bin_name` is sourced from `Cli::command().get_name()` (a static string
literal `"pirs"`), not from `argv[0]`. There is no path-traversal vector.
No change.

### Low — FIXED

> `tests/fixtures_parse.rs:97-107` — Missing private-key prefix in secret
> denylist.

Added `-----BEGIN PRIVATE KEY`, `-----BEGIN RSA PRIVATE KEY`,
`-----BEGIN OPENSSH PRIVATE KEY`, and `-----BEGIN EC PRIVATE KEY` to the
needle list. Fixture corpus tests still pass.

### Low — accepted

> `commands/completions.rs:36` — Unverified `clap_complete` filename
> sanitisation; no canonicalised-child check on the returned path.

`clap_complete` produces fixed canonical filenames per shell (`_pirs`,
`pirs.bash`, `pirs.fish`, `_pirs.ps1`, `pirs.elv`) and never derives the
filename from user input. Adding a canonicalised-child assertion would be
defence-in-depth but does not address a realistic threat for a local CLI.
No change.

### Informational — documented

1. **Redaction scope ambiguity** in `book/src/mcp-security.md` — the
   chapter already states redaction applies only to a named field list and
   is best-effort; no change required, but worth re-reading on every
   security PR.
2. **No fixture exercises real-format synthetic tokens.** Out of scope for
   this PR — redaction-pipeline integration tests live in
   `crates/pirs-core/src/export.rs` unit tests.
3. **ACL enforcement deferred for `Confidential`/`Restricted` PIRs.**
   Already acknowledged in the MCP-security chapter as future work.

## Strengths called out by reviewers

- Idiomatic `clap::CommandFactory` use in the dispatch arm.
- Fixture corpus covers every `IncidentType` with realistic-but-synthetic
  content; no PII or real-format secrets detected.
- `concat!()` trick in `req_fix_003` prevents the test file itself from
  triggering credential scanners.
- All six required documentation chapters present and cross-linked.
- ADR-0009/0010/0011 traceably map decisions to spec requirements.

## Net code changes from review

- `book/src/agent-workflows.md` — Pattern 1 example corrected.
- `crates/pirs-core/tests/fixtures_parse.rs` — denylist extended with PEM
  private-key prefixes.

All tests still green:

```text
test result: ok. 1 passed
test result: ok. 25 passed   (CLI integration including new req_comp_*)
test result: ok. 4 passed
test result: ok. 19 passed
test result: ok. 3 passed    (fixture smoke tests)
```

`cargo clippy --all-targets --all-features -- -D warnings` clean.
`mdbook build book` clean (no warnings).

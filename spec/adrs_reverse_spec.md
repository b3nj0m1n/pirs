# Reverse-Engineered Specification: `adrs`

> **Source**: `/Users/ben/IdeaProjects/adrs` (workspace version `0.7.3`)
> **Method**: Static code analysis using the **Spec Miner** workflow (Glob/Grep/Read).
> **Format**: Observations expressed in [EARS](https://alistairmavin.com/ears/) syntax with Given/When/Then acceptance criteria.
> All observations are grounded in source code; file paths and line ranges are cited inline. Items that could not be verified from code are listed under [Uncertainties](#7-uncertainties-and-questions). The methodological and behavioural assumptions on which this whole document rests are listed in [§9 Assumption Inventory](#9-assumption-inventory).

> ⚠️ **Read this first — descriptive vs prescriptive.**
> Every "shall" in §4 is a *reverse-engineered observation* of what the code does today, not a normative requirement the maintainers ratified. Treat this spec as a **mirror of current behaviour**, not as a contract. Behaviours flagged in §9 as Low-confidence assumptions must be confirmed before being relied upon by downstream consumers, ports, or alternate implementations.

---

## 1. Overview

`adrs` is a Rust CLI for creating and managing **Architecture Decision Records (ADRs)**, with optional Model Context Protocol (MCP) server support for AI-agent integration.

It is designed to be:

- **Backward compatible** with the de facto-standard [`npryce/adr-tools`](https://github.com/npryce/adr-tools) repositories (a `.adr-dir` file + `NNNN-slug.md` files).
- **Forward compatible** via a "NextGen" mode that introduces YAML frontmatter, tags, and a TOML config file (`adrs.toml`).
- **Format-agnostic**: supports the classic Michael Nygard template *and* MADR 4.0.0.
- **Library-first**: business logic lives in [`adrs-core`](crates/adrs-core/src/lib.rs); the CLI binary and MCP server are thin frontends over it.

The architecture intent is recorded in the project's own ADRs:
[ADR 0004 (Library-first)](doc/adr/0004-library-first-architecture.md),
[ADR 0005 (Compatible / NextGen dual mode)](doc/adr/0005-dual-mode-compatible-and-nextgen.md),
[ADR 0006 (YAML frontmatter)](doc/adr/0006-yaml-frontmatter-for-metadata.md),
[ADR 0007 (minijinja templates)](doc/adr/0007-use-minijinja-for-templates.md),
[ADR 0008 (tower-mcp)](doc/adr/0008-use-tower-mcp-for-mcp-server.md).

---

## 2. Architecture Summary

### 2.1 Technology Stack

| Concern | Choice | Source |
|---|---|---|
| Language / edition | Rust, edition 2024 | [Cargo.toml](Cargo.toml#L4-L8) |
| CLI parsing | `clap` 4 (derive) | [Cargo.toml](Cargo.toml#L29-L31) |
| Errors | `thiserror` (lib), `anyhow` (bin) | [Cargo.toml](Cargo.toml#L13-L33) |
| Serialization | `serde`, `serde_yaml` 0.9, `toml` 0.8 | [Cargo.toml](Cargo.toml#L14-L18) |
| Markdown | `pulldown-cmark`, `pulldown-cmark-to-cmark` | [Cargo.toml](Cargo.toml#L20-L22) |
| Templates | `minijinja` 2 (with `loader`) | [Cargo.toml](Cargo.toml#L24-L25) |
| Filesystem walk | `walkdir` 2.5 | [Cargo.toml](Cargo.toml#L27-L28) |
| Date / time | `time` 0.3 | [Cargo.toml](Cargo.toml#L30-L31) |
| Fuzzy search | `fuzzy-matcher` 0.3 | [Cargo.toml](Cargo.toml#L33-L34) |
| Editor spawn | `edit` 0.1 | [Cargo.toml](Cargo.toml#L36-L37) |
| MCP server | `tower-mcp` 0.9 (+ `http`), `tokio` | [Cargo.toml](Cargo.toml#L40-L44) |
| Linting | `mdbook-lint-core` + `mdbook-lint-rulesets::adr` | [crates/adrs-core/src/lint.rs](crates/adrs-core/src/lint.rs) |
| Testing | `assert_cmd`, `assert_fs`, `predicates`, `tempfile`, `proptest`, `serial_test`, `test-case` | [Cargo.toml](Cargo.toml#L48-L55) |

### 2.2 Workspace Layout

```
adrs/
├── crates/
│   ├── adrs/              # CLI binary + MCP server (thin frontend)
│   │   ├── src/
│   │   │   ├── main.rs    # clap dispatch, global flags --ng / -C
│   │   │   ├── mcp.rs     # MCP server (stdio + optional HTTP)
│   │   │   └── commands/  # one file per subcommand
│   │   └── tests/         # CLI integration (cli.rs, scenarios.rs)
│   └── adrs-core/         # Library: all business logic
│       ├── src/
│       │   ├── config.rs      # discovery, .adr-dir / adrs.toml
│       │   ├── doctor.rs      # legacy health-check (deprecated)
│       │   ├── error.rs       # thiserror error enum
│       │   ├── export.rs      # JSON-ADR v1 export/import
│       │   ├── lib.rs         # public API surface
│       │   ├── lint.rs        # mdbook-lint-rulesets::adr
│       │   ├── parse.rs       # dual-mode parser (YAML FM vs legacy)
│       │   ├── repository.rs  # CRUD over ADR collection
│       │   ├── template.rs    # minijinja templates, formats/variants
│       │   └── types.rs       # Adr, AdrStatus, AdrLink, slug rules
│       └── tests/             # adr_tools_compat, edge_cases, real_world_corpus
├── schema/json-adr/v1.json    # JSON Schema for the export format
├── doc/adr/                   # The project's own ADRs (dogfooding)
├── book/                      # mdBook user manual
├── Dockerfile                 # ghcr.io/joshrotenberg/adrs
└── tests/fixtures/adr-corpus/ # Real-world ADR fixtures
```

### 2.3 Data Flow

```
       CLI args ──► clap ──► subcommand handler ──┐
                                                   │
                                                   ▼
                              Config::discover(cwd) ──► .adr-dir | adrs.toml | $ADRS_CONFIG | ~/.config/adrs/config.toml | defaults
                                                   │
                                                   ▼
                                       Repository::open(root)
                                                   │
                          ┌────────────────────────┼─────────────────────────┐
                          ▼                        ▼                         ▼
                       Parser                  walkdir                   Template
                  (YAML FM ↔ MD)        (NNNN-*.md, depth=1)          (minijinja)
                          │                        │                         │
                          └────────────────────────┴─────────────────────────┘
                                                   │
                                                   ▼
                                    stdout / file writes / exit code

MCP path:                                                            
       client (stdio|HTTP) ──► tower-mcp router ──► AdrState{root} ──► Repository::open(root) per call
```

---

## 3. Module / Directory Structure

| Module | Responsibility |
|---|---|
| [`adrs-core/src/types.rs`](crates/adrs-core/src/types.rs) | `Adr`, `AdrStatus`, `AdrLink`, `LinkKind`, slug & filename rules, flexible YAML deserialization (string-or-vec). |
| [`adrs-core/src/parse.rs`](crates/adrs-core/src/parse.rs) | Detects YAML frontmatter (`---\n…\n---`) vs legacy markdown; extracts H1 title and H2 sections (Status, Context, Decision, Consequences). |
| [`adrs-core/src/repository.rs`](crates/adrs-core/src/repository.rs) | `list`, `get`, `find`, `next_number`, `create`, `supersede`, `link`, `set_status`, surgical `update_metadata`. |
| [`adrs-core/src/template.rs`](crates/adrs-core/src/template.rs) | minijinja env + `pad` filter; built-in Nygard / MADR templates × Full / Minimal / Bare / BareMinimal variants; custom template loading. |
| [`adrs-core/src/config.rs`](crates/adrs-core/src/config.rs) | Upward search for `adrs.toml` / `.adr-dir`; `$ADRS_CONFIG`, `$ADR_DIRECTORY`; modes: `Compatible` / `NextGen`. |
| [`adrs-core/src/lint.rs`](crates/adrs-core/src/lint.rs) | Unified linting (rules `ADR001`–`ADR017`) replacing `doctor.rs`. |
| [`adrs-core/src/doctor.rs`](crates/adrs-core/src/doctor.rs) | Legacy health checks (deprecated since 0.6.0). |
| [`adrs-core/src/export.rs`](crates/adrs-core/src/export.rs) | `JsonAdr`, `JsonAdrBulkExport`, import options & result types (v1.0.0). |
| [`adrs-core/src/error.rs`](crates/adrs-core/src/error.rs) | `Error` enum via `thiserror`. |
| [`adrs/src/main.rs`](crates/adrs/src/main.rs) | `clap` dispatch; global `--ng`, `-C/--cwd`. |
| [`adrs/src/commands/*.rs`](crates/adrs/src/commands/) | One file per subcommand. |
| [`adrs/src/mcp.rs`](crates/adrs/src/mcp.rs) | MCP server (stdio default; HTTP behind `mcp-http` feature). |

---

## 4. Observed Functional Requirements (EARS)

> Each requirement is grounded in source code. Identifiers follow `OBS-<AREA>-<NNN>`.

### 4.1 Configuration & Discovery

**OBS-CFG-001 — Configuration discovery order**
When the CLI starts, the system shall locate configuration in the following order: (1) `$ADRS_CONFIG` if set, (2) the nearest `adrs.toml` walking upward from `--cwd` / current directory, (3) the nearest `.adr-dir`, (4) `~/.config/adrs/config.toml` (global), (5) built-in defaults (`adr_dir = "doc/adr"`, `mode = Compatible`).
Source: [config.rs](crates/adrs-core/src/config.rs)

> Verification note (2026-04-25): static code inspection confirms the broad order, but the project-config search is level-wise: at each directory level `adrs.toml` is checked before `.adr-dir`, then discovery moves to the parent. A nearer `.adr-dir` therefore wins over an `adrs.toml` higher in the tree.

**OBS-CFG-002 — Environment override**
Where `$ADR_DIRECTORY` is set, the system shall use it as the effective `adr_dir`, overriding the config file value.
Source: [config.rs](crates/adrs-core/src/config.rs)

**OBS-CFG-003 — Mode selection**
While the global `--ng` flag is supplied or `mode = "ng"` is present in `adrs.toml`, the system shall operate in NextGen mode; otherwise it shall operate in Compatible mode.
Source: [main.rs](crates/adrs/src/main.rs), [config.rs](crates/adrs-core/src/config.rs)

**OBS-CFG-004 — Compatible-mode config artifact**
While operating in Compatible mode, when `init` is run, the system shall create a `.adr-dir` file containing the ADR directory path (single line).
Source: [init.rs](crates/adrs/src/commands/init.rs), [config.rs](crates/adrs-core/src/config.rs)

**OBS-CFG-005 — NextGen-mode config artifact**
While operating in NextGen mode, when `init` is run, the system shall create an `adrs.toml` file containing `adr_dir`, `mode = "ng"` and an optional `[templates]` section.
Source: [init.rs](crates/adrs/src/commands/init.rs), [config.rs](crates/adrs-core/src/config.rs)

**OBS-CFG-006 — Working-directory override**
When `-C <DIR>` / `--cwd <DIR>` is supplied, the system shall perform configuration discovery and all filesystem operations relative to `<DIR>` instead of the process CWD.
Source: [main.rs](crates/adrs/src/main.rs)

### 4.2 `init`

**OBS-INIT-001 — Idempotent directory creation**
When `adrs init [DIRECTORY]` runs and the target directory does not exist, the system shall create it (default `doc/adr`).
Source: [init.rs](crates/adrs/src/commands/init.rs)

**OBS-INIT-002 — Bootstrap ADR**
When `init` runs against an empty ADR directory, the system shall create ADR `0001` titled "Record architecture decisions" with status `Accepted`.
Source: [init.rs](crates/adrs/src/commands/init.rs), [repository.rs](crates/adrs-core/src/repository.rs)

**OBS-INIT-003 — Non-empty preservation**
When `init` runs against a directory that already contains ADRs, the system shall preserve them and report the existing count to stdout.
Source: [init.rs](crates/adrs/src/commands/init.rs)

### 4.3 `new`

**OBS-NEW-001 — Sequential numbering**
When `adrs new "<TITLE>"` runs, the system shall assign `next_number = max(existing) + 1` (or `1` if none) and write `NNNN-<slug>.md` into the ADR directory.
Source: [repository.rs](crates/adrs-core/src/repository.rs), [types.rs](crates/adrs-core/src/types.rs)

**OBS-NEW-002 — Slug rules**
The system shall produce slugs that are lowercase, contain only `[a-z0-9-]`, never start or end with `-`, and never contain consecutive `-` (verified via `proptest`).
Source: [types.rs](crates/adrs-core/src/types.rs), [adrs-core/proptest-regressions/types.txt](crates/adrs-core/proptest-regressions/types.txt)

**OBS-NEW-003 — Default status**
When `--status` is not supplied, the system shall set the new ADR's status to `Proposed`.
Source: [new.rs](crates/adrs/src/commands/new.rs)

**OBS-NEW-004 — Bidirectional supersede**
When `--supersedes N` is supplied, the system shall (a) add a `Supersedes` link from the new ADR to ADR N, (b) set ADR N's status to `Superseded`, and (c) add a `SupersededBy` link from ADR N back to the new ADR. Both files shall be persisted.
Source: [repository.rs](crates/adrs-core/src/repository.rs)

**OBS-NEW-005 — Custom links**
When `--link TARGET:KIND:REVERSE_KIND` is supplied, the system shall create a bidirectional link of the given kinds between the new ADR and TARGET.
Source: [new.rs](crates/adrs/src/commands/new.rs), [repository.rs](crates/adrs-core/src/repository.rs)

**OBS-NEW-006 — Tags require NextGen**
When `--tags` is supplied while not in NextGen mode, the system shall return an error and shall not create the ADR.
Source: [new.rs](crates/adrs/src/commands/new.rs)

**OBS-NEW-007 — Editor invocation**
While `--no-edit` is not set, when the new ADR file has been written, the system shall open it in `$EDITOR` (or platform fallback) via the `edit` crate.
Source: [new.rs](crates/adrs/src/commands/new.rs)

**OBS-NEW-008 — Format / variant override**
Where `--format` (`nygard`|`madr`) or `--variant` (`full`|`minimal`|`bare`|`bare-minimal`) is supplied, the system shall use the corresponding built-in template, overriding `[templates]` config.
Source: [template.rs](crates/adrs-core/src/template.rs), [new.rs](crates/adrs/src/commands/new.rs)

### 4.4 `edit`, `list`, `search`

**OBS-EDIT-001 — Lookup by number or fuzzy title**
When `adrs edit <QUERY>` is invoked and `<QUERY>` parses as `u32`, the system shall look up by exact ADR number; otherwise it shall fuzzy-match (`SkimMatcherV2`) against titles.
Source: [repository.rs](crates/adrs-core/src/repository.rs)

**OBS-EDIT-002 — Ambiguity guard**
When fuzzy matching produces a top score that is not at least 2× the second-best score, the system shall return an `AmbiguousAdr` error listing up to 5 candidates instead of opening any file.
Source: [repository.rs](crates/adrs-core/src/repository.rs)

**OBS-LIST-001 — Sorted listing**
When `adrs list` runs, the system shall return ADRs sorted by ascending number.
Source: [repository.rs](crates/adrs-core/src/repository.rs)

**OBS-LIST-002 — Filters**
When any of `--status`, `--since`, `--until`, `--decider`, `--tag` are supplied, the system shall apply them as conjunctive filters before output.
Source: [list.rs](crates/adrs/src/commands/list.rs)

**OBS-LIST-003 — Tag filter requires NextGen**
When `--tag` is supplied while not in NextGen mode, the system shall return an error.
Source: [list.rs](crates/adrs/src/commands/list.rs)

> Verification note (2026-04-25): contradicted by static code inspection. `commands::list` receives only `root` and filter arguments, opens the repository, and applies the tag filter unconditionally; `main.rs` discovers config for `list` but does not pass mode information into the command. Current behaviour is: `adrs list --tag <TAG>` filters by tag in both Compatible and NextGen mode.

**OBS-LIST-004 — Output formats**
Where `--long`/`-l` is supplied, the system shall print `<number> [<status>] <date> <title>` per line; otherwise it shall print one file path per line.
Source: [list.rs](crates/adrs/src/commands/list.rs)

**OBS-SRCH-001 — Default scope**
When `adrs search <QUERY>` runs without `--title`, the system shall search title plus Context, Decision and Consequences sections; with `--title`/`-t` it shall search titles only.
Source: [search.rs](crates/adrs/src/commands/search.rs)

**OBS-SRCH-002 — Case sensitivity**
While `--case-sensitive`/`-c` is not set, the system shall perform case-insensitive matching.
Source: [search.rs](crates/adrs/src/commands/search.rs)

**OBS-SRCH-003 — Snippet output**
When a match is found, the system shall print the ADR number, title, matching section, and a context snippet (~40 chars before/after the match, bounded by word edges).
Source: [search.rs](crates/adrs/src/commands/search.rs)

**OBS-SRCH-004 — No-match message**
When no ADRs match, the system shall print `No matches found for '<QUERY>'`.
Source: [search.rs](crates/adrs/src/commands/search.rs)

### 4.5 `link`, `status`

**OBS-LINK-001 — Bidirectional link write**
When `adrs link <SRC> <KIND> <TGT> [<REV>]` runs, the system shall write a forward link from SRC to TGT and a reverse link from TGT to SRC; if `<REV>` is omitted it shall be derived (`Supersedes↔SupersededBy`, `Amends↔AmendedBy`, `RelatesTo↔RelatesTo`, `Custom(s)↔Custom(s)`).
Source: [types.rs](crates/adrs-core/src/types.rs), [repository.rs](crates/adrs-core/src/repository.rs)

**OBS-STAT-001 — Status change**
When `adrs status <ADR> <STATUS>` runs, the system shall set the ADR's status, persisting via a surgical metadata update that preserves body content.
Source: [status.rs](crates/adrs/src/commands/status.rs), [repository.rs](crates/adrs-core/src/repository.rs)

**OBS-STAT-002 — `--by` validity**
When `--by N` is supplied, the system shall require `<STATUS>` to be `superseded`; otherwise it shall return a validation error.
Source: [status.rs](crates/adrs/src/commands/status.rs)

**OBS-STAT-003 — `--by` link addition**
When `<STATUS> = superseded` and `--by N` is supplied, the system shall validate ADR N exists and add a `SupersededBy` link if absent.
Source: [repository.rs](crates/adrs-core/src/repository.rs)

### 4.6 `config`

**OBS-CONF-001 — Config dump**
When `adrs config` runs, the system shall print: project root, config source (`Project`/`Global`/`Environment`/`Default`), `adr_dir` (relative + resolved), mode, and `[templates]` settings if present.
Source: [config.rs (cmd)](crates/adrs/src/commands/config.rs), [config.rs (lib)](crates/adrs-core/src/config.rs)

### 4.7 `doctor`

**OBS-DOC-001 — Linting**
When `adrs doctor` runs, the system shall execute the unified `mdbook-lint-rulesets::adr` rule set (rules `ADR001`–`ADR017`) plus repository-level checks for duplicate numbers, numbering gaps, broken links, and superseded-without-`SupersededBy`.
Source: [lint.rs](crates/adrs-core/src/lint.rs), [doctor.rs (cmd)](crates/adrs/src/commands/doctor.rs)

**OBS-DOC-002 — Issue formatting**
When issues are found, the system shall print each issue as `<severity>: [<rule_id>] <message> [<path>:<line>]` followed by a summary `Found N error(s), M warning(s), K info(s)`.
Source: [doctor.rs (cmd)](crates/adrs/src/commands/doctor.rs)

**OBS-DOC-003 — Exit code**
When at least one Error-severity issue is reported, the system shall exit with code `1`; warnings and info do not affect the exit code.
Source: [doctor.rs (cmd)](crates/adrs/src/commands/doctor.rs)

### 4.8 `generate`

**OBS-GEN-001 — TOC**
When `adrs generate toc` runs, the system shall print a markdown table of contents linking each ADR; `--ordered` produces a numbered list, `--prefix` is prepended to link targets, and `--intro`/`--outro` files are concatenated before/after the list.
Source: [generate.rs](crates/adrs/src/commands/generate.rs)

**OBS-GEN-002 — Graphviz**
When `adrs generate graph` runs, the system shall print a `digraph` with one node per ADR (ID `_<number>`), dotted weighted edges between consecutive numbers, and labelled solid edges for explicit links; `--prefix` and `--extension` parameterize node URLs.
Source: [generate.rs](crates/adrs/src/commands/generate.rs)

**OBS-GEN-003 — mdBook**
When `adrs generate book <OUTPUT>` runs, the system shall create `<OUTPUT>/book.toml` and `<OUTPUT>/src/SUMMARY.md` and copy each ADR markdown file into `<OUTPUT>/src/`. Default author is `whoami::username()`; default title is "Architecture Decision Records".
Source: [generate.rs](crates/adrs/src/commands/generate.rs)

### 4.9 `export` / `import`

**OBS-EXP-001 — JSON-ADR v1.0.0**
When `adrs export json` runs, the system shall emit JSON conforming to `schema/json-adr/v1.json` (version `1.0.0`).
Source: [export.rs](crates/adrs-core/src/export.rs), [v1.json](schema/json-adr/v1.json)

**OBS-EXP-002 — Single vs bulk**
When `<adr_number>` is supplied, the system shall emit a single `JsonAdr` object; otherwise it shall emit a `JsonAdrBulkExport` containing tool/version metadata, `generated_at` timestamp, repository info, and an `adrs` array.
Source: [export.rs](crates/adrs-core/src/export.rs)

**OBS-EXP-003 — Metadata-only**
Where `--metadata-only` is supplied, the system shall omit `context`, `decision`, `consequences`, `confirmation`, `decision_drivers`, `considered_options`, and `custom_sections` and shall populate `source_uri` from `--base-url + filename` if `--base-url` is given.
Source: [export.rs](crates/adrs-core/src/export.rs)

**OBS-EXP-004 — `--dir` vs repository**
When `--dir <PATH>` is supplied, the system shall export ADRs from `<PATH>` without requiring `adrs.toml` / `.adr-dir`; supplying both `--dir` and `<adr_number>` shall be rejected.
Source: [export.rs (cmd)](crates/adrs/src/commands/export.rs)

**OBS-IMP-001 — Source**
When `adrs import json <FILE>` runs, the system shall accept a filesystem path or `-` (stdin) as input.
Source: [import.rs (cmd)](crates/adrs/src/commands/import.rs)

**OBS-IMP-002 — Renumbering**
Where `--renumber` is supplied, the system shall reassign sequential numbers and rewrite link targets that reference imported ADRs; links to ADRs outside the import set shall produce warnings.
Source: [import.rs (cmd)](crates/adrs/src/commands/import.rs), [export.rs](crates/adrs-core/src/export.rs)

**OBS-IMP-003 — Overwrite policy**
When a target file already exists, the system shall skip it unless `--overwrite` is supplied.
Source: [import.rs (cmd)](crates/adrs/src/commands/import.rs)

**OBS-IMP-004 — Dry run**
Where `--dry-run` is supplied, the system shall report what would be imported and shall not write any files.
Source: [import.rs (cmd)](crates/adrs/src/commands/import.rs)

### 4.10 `template`

**OBS-TMPL-001 — Listing**
When `adrs template list` runs, the system shall print the names of built-in formats (`nygard`, `madr`) and variants (`full`, `minimal`, `bare`, `bare-minimal`) with descriptions.
Source: [template.rs (cmd)](crates/adrs/src/commands/template.rs)

**OBS-TMPL-002 — Show**
When `adrs template show <FORMAT> [--variant <V>]` runs, the system shall print the variable reference followed by the raw template body.
Source: [template.rs (cmd)](crates/adrs/src/commands/template.rs)

**OBS-TMPL-003 — Custom templates**
Where `templates.custom = "<PATH>"` is set in `adrs.toml`, the system shall load the template from `<PATH>` for `new`, unless overridden by `--format`/`--variant`.
Source: [template.rs](crates/adrs-core/src/template.rs), [config.rs](crates/adrs-core/src/config.rs)

**OBS-TMPL-004 — `pad` filter**
The template engine shall expose a `pad` filter that zero-pads integers (default width 4): `{{ number | pad }} → "0042"`, `{{ number | pad(width=6) }} → "000042"`.
Source: [template.rs](crates/adrs-core/src/template.rs)

### 4.11 MCP Server

**OBS-MCP-001 — Transports**
When `adrs mcp` runs without `--http`, the system shall serve MCP over stdio. Where the `mcp-http` feature is enabled and `--http <ADDR>` is supplied, the system shall serve MCP over HTTP at `ADDR/mcp`.
Source: [mcp.rs](crates/adrs/src/mcp.rs)

> Verification note (2026-04-25): transport behaviour is confirmed, but the clap command is `adrs mcp serve`; bare `adrs mcp` does not start the server.

**OBS-MCP-002 — Per-call repository**
When an MCP tool is invoked, the system shall open the repository fresh from `AdrState.root` for that call (no shared mutable state).
Source: [mcp.rs](crates/adrs/src/mcp.rs)

**OBS-MCP-003 — Read-only tools**
The MCP server shall expose the read-only tools `list_adrs`, `get_adr`, `search_adrs`, `get_repository_info`, `get_related_adrs`, `validate_adr`, `get_adr_sections`, `compare_adrs`, `suggest_tags`.
Source: [mcp.rs](crates/adrs/src/mcp.rs)

**OBS-MCP-004 — Write tools**
The MCP server shall expose the write tools `create_adr`, `update_status`, `link_adrs`, `update_content`, `update_tags`, `bulk_update_status`.
Source: [mcp.rs](crates/adrs/src/mcp.rs)

**OBS-MCP-005 — Tag tool gating**
When `update_tags` is invoked while the repository is in Compatible mode, the system shall return an error result.
Source: [mcp.rs](crates/adrs/src/mcp.rs), [new.rs](crates/adrs/src/commands/new.rs)

### 4.12 Parsing & Compatibility

**OBS-PARSE-001 — Frontmatter detection**
When parsing a file beginning with `---\n`, the system shall extract a YAML frontmatter block terminated by `\n---\n` and parse it as ADR metadata; otherwise it shall parse legacy markdown.
Source: [parse.rs](crates/adrs-core/src/parse.rs)

**OBS-PARSE-002 — Section extraction**
When parsing legacy markdown, the system shall recognize H2 sections `Status`, `Context`, `Decision`, and `Consequences` (case-insensitive).
Source: [parse.rs](crates/adrs-core/src/parse.rs)

**OBS-PARSE-003 — adr-tools typo tolerance**
When parsing a status value, the system shall accept the misspelling `superceded` as `Superseded`.
Source: [types.rs](crates/adrs-core/src/types.rs), [adr_tools_compat.rs](crates/adrs-core/tests/adr_tools_compat.rs)

**OBS-PARSE-004 — Flexible scalar/array fields**
When deserializing YAML fields `decision-makers`, `consulted`, `informed`, `tags`, the system shall accept either a single string or an array of strings.
Source: [types.rs](crates/adrs-core/src/types.rs)

---

## 5. Observed Non-Functional Requirements

### 5.1 Compatibility

- The system shall remain a drop-in replacement for `npryce/adr-tools` repositories: `.adr-dir` config, `NNNN-slug.md` files, plain markdown `## Status` section. Verified by [adr_tools_compat.rs](crates/adrs-core/tests/adr_tools_compat.rs).
- The system shall accept Compatible-mode files when running in NextGen mode and vice versa (mode controls *writes*, not *reads*).

### 5.2 Security / Robustness

- ADR filenames are derived through a slug pipeline that strips all non-`[a-z0-9-]` characters, mitigating path injection from user titles. Source: [types.rs](crates/adrs-core/src/types.rs).
- The ADR directory is always resolved relative to a discovered project root; user input does not produce absolute paths.
- Editor invocation goes through the `edit` crate, which avoids shell interpolation of `$EDITOR`.
- YAML deserialization is strongly typed (struct-based, not `serde_yaml::Value`), reducing surface for type-confusion attacks.
- **Gap:** The MCP HTTP transport has no observed authentication or authorization layer.
- **Gap:** `next_number()` is racy under concurrent invocations (no file lock).
- **Gap:** No backups are taken before destructive metadata rewrites.

### 5.3 Performance

- All repository operations are synchronous and load entire ADR collections into memory (`walkdir` + parse loop in [`Repository::list`](crates/adrs-core/src/repository.rs)).
- File traversal is bounded to `max_depth = 1` in the ADR directory.
- Async runtime (`tokio`) is used only by the MCP server.
- No use of `rayon` or other parallelism for filesystem operations.

### 5.4 Cross-Platform

- Pure `std::path::PathBuf` usage; no hard-coded path separators.
- No explicit line-ending handling — relies on Rust stdlib.
- Distribution covers macOS, Linux, Windows binaries plus Docker (`Dockerfile`), per [README.md](README.md).

### 5.5 Error Handling

The library exposes a `thiserror` enum from [error.rs](crates/adrs-core/src/error.rs):

| Variant | Trigger |
|---|---|
| `AdrDirNotFound` | No config found by `discover()` |
| `AdrDirExists(path)` | `init` over existing directory (when checked) |
| `AdrNotFound(query)` | `find` / `get` failed |
| `AmbiguousAdr { query, matches }` | Fuzzy match top-score not 2× runner-up |
| `InvalidNumber(s)` | Parse failure |
| `InvalidFormat { path, reason }` | Malformed frontmatter / sections |
| `MissingField { path, field }` | Required frontmatter field missing |
| `InvalidStatus(s)` | Reserved (current parser is infallible/permissive) |
| `InvalidLink(s)` | `--link TARGET:KIND:REVERSE` parse failure |
| `TemplateNotFound(s)` / `TemplateError(s)` | Unknown / failing template |
| `ConfigError(s)` | Config parse/validation |
| `Io(e)` / `Yaml(e)` / `Toml(e)` | I/O & format passthroughs |

CLI exit codes (observed):

| Code | Condition |
|---|---|
| `0` | Command succeeded |
| `1` | Any unhandled error (anyhow → main); `doctor` Error-severity issues found |

### 5.6 Testing

| Test surface | Location |
|---|---|
| CLI integration scenarios | [crates/adrs/tests/scenarios.rs](crates/adrs/tests/scenarios.rs), [crates/adrs/tests/cli.rs](crates/adrs/tests/cli.rs) |
| `adr-tools` compatibility | [crates/adrs-core/tests/adr_tools_compat.rs](crates/adrs-core/tests/adr_tools_compat.rs) |
| Edge cases | [crates/adrs-core/tests/edge_cases.rs](crates/adrs-core/tests/edge_cases.rs) |
| Real-world corpus | [crates/adrs-core/tests/real_world_corpus.rs](crates/adrs-core/tests/real_world_corpus.rs) + [tests/fixtures/adr-corpus](tests/fixtures/adr-corpus) |
| Property tests (slug) | [crates/adrs-core/proptest-regressions/types.txt](crates/adrs-core/proptest-regressions/types.txt) |

---

## 6. Inferred Acceptance Criteria (Given / When / Then)

### AC-001 — Initialize a fresh repository (Compatible mode)
- **Given** an empty directory with no `.adr-dir` and no `adrs.toml`
- **When** the user runs `adrs init`
- **Then** the system creates `doc/adr/`, writes a `.adr-dir` file containing `doc/adr`, creates `0001-record-architecture-decisions.md` with status `Accepted`, and prints the ADR directory path.

### AC-002 — Initialize in NextGen mode
- **Given** an empty directory
- **When** the user runs `adrs --ng init`
- **Then** the system creates `doc/adr/` and writes `adrs.toml` containing `mode = "ng"` and the ADR directory.

### AC-003 — Create a new ADR with default options
- **Given** an initialized repository with one existing ADR
- **When** the user runs `adrs new "Use PostgreSQL for persistence" --no-edit`
- **Then** the system writes `doc/adr/0002-use-postgresql-for-persistence.md` with status `Proposed` and prints the new file path.

### AC-004 — Supersede an existing ADR
- **Given** ADR `0002` exists with status `Accepted`
- **When** the user runs `adrs new "Use MySQL" --supersedes 2 --no-edit`
- **Then** ADR `0003` is created with a `Supersedes → 2` link, ADR `0002` is updated to status `Superseded` with a `SupersededBy → 3` link, and both files are persisted.

### AC-005 — Tags require NextGen mode
- **Given** a repository in Compatible mode
- **When** the user runs `adrs new "Test" --tags foo,bar`
- **Then** the system exits non-zero with an error indicating tags require NextGen mode and no file is created.

### AC-006 — Fuzzy edit lookup with ambiguity
- **Given** ADRs whose titles fuzzy-match `"use"` with comparable scores
- **When** the user runs `adrs edit use`
- **Then** the system returns an `AmbiguousAdr` error listing up to 5 candidates and does not open any editor.

### AC-007 — Filtered listing
- **Given** a repository with mixed-status ADRs
- **When** the user runs `adrs list --status accepted --long`
- **Then** the system prints only `Accepted` ADRs in the long format `<number> [<status>] <date> <title>`, sorted ascending by number.

### AC-008 — Doctor finds duplicate numbers
- **Given** two files `0002-a.md` and `0002-b.md` exist in the ADR directory
- **When** the user runs `adrs doctor`
- **Then** the system reports an `Error` issue (rule `ADR010` or equivalent) for the duplicate and exits with code `1`.

### AC-009 — Round-trip via JSON-ADR
- **Given** an initialized repository with several ADRs
- **When** the user runs `adrs export json --pretty > adrs.json` and then `adrs import json adrs.json --dry-run` against an empty target
- **Then** the system reports the same set of ADRs would be imported with no warnings about broken cross-references.

### AC-010 — Generate Graphviz with link edges
- **Given** ADRs `0001`, `0002`, `0003` where `0003 Supersedes 0002`
- **When** the user runs `adrs generate graph`
- **Then** stdout contains `digraph {`, dotted edges `_1 -> _2 -> _3`, and a labelled solid edge `_3 -> _2 [label="Supersedes"]` (or similar).

### AC-011 — MCP `update_tags` requires NextGen
- **Given** an MCP session against a Compatible-mode repository
- **When** the client calls `update_tags { number: 1, tags: ["x"] }`
- **Then** the response is an error result and no file is modified.

### AC-012 — Custom template precedence
- **Given** `adrs.toml` with `[templates] custom = "templates/x.md"`
- **When** the user runs `adrs new "Y" --format madr --no-edit`
- **Then** the system uses the built-in MADR template (CLI flag wins), not `templates/x.md`.

### AC-013 — `--cwd` redirects discovery
- **Given** repository `/proj` initialized and an unrelated CWD `/tmp`
- **When** the user runs `adrs -C /proj list`
- **Then** the system lists ADRs from `/proj/doc/adr` regardless of process CWD.

---

## 7. Uncertainties and Questions

- [ ] **Concurrent writes** — `next_number()` has no locking. What is the intended behaviour under parallel `adrs new` / MCP `create_adr` calls?
- [ ] **HTTP MCP auth** — Should the HTTP transport support tokens / TLS, or is it intentionally LAN-only?
- [ ] **Frontmatter extension** — Are unknown YAML keys preserved on round-trip, or silently dropped? (No `#[serde(flatten)]` or `extras` field observed.)
- [ ] **`InvalidStatus` reachability** — The error variant exists, but status parsing appears infallible (`Custom(s)` accepts any string). Is the variant dead code?
- [ ] **Doctor exit code on warnings only** — Is the convention "errors only fail" intentional or a bug?
- [ ] **`init` over non-empty dir** — `AdrDirExists` exists in the error enum; does `init` actually emit it, or does it silently absorb existing ADRs (per OBS-INIT-003)?
- [ ] **Renumber semantics** — When `--renumber` collides with `--overwrite=false`, which wins?
- [ ] **Git integration** — None observed; intentional?
- [ ] **Template inheritance** — Can custom minijinja templates `{% extends %}` the built-ins? Loader is configured but built-ins are inlined as constants.
- [ ] **MADR write fidelity** — Are `decision_drivers` / `considered_options` round-trippable through write → read → write?

---

## 8. Recommendations

1. **Add file locking** around `next_number()` + create to prevent number collisions in concurrent invocations (CLI and MCP).
2. **Authenticate the MCP HTTP transport** (bearer token, optional TLS) before recommending it for non-loopback use.
3. **Document and test exit codes** explicitly for every subcommand (currently observed only for `doctor`).
4. **Persist unknown YAML keys** via `#[serde(flatten)] extras: BTreeMap<String, Value>` so external tooling can annotate ADRs without losing data on a write.
5. **Replace deprecated `doctor.rs`** completely or remove it from the library surface; today both `lint` and `doctor` modules are publicly re-exported.
6. **Validate links in `link` / `new --link`** at write time, not only via `doctor`, so dangling references are caught immediately.
7. **Provide a `--no-color` / `NO_COLOR` audit** — confirm output is pipeline-friendly across all subcommands.
8. **Add an MCP `delete_adr` audit trail** if/when destructive tools are added; currently no destructive write tools exist.
9. **Backup-before-rewrite** option (or rely on git) for `update_metadata`, `link`, `status`, `update_content`.
10. **Surface `MissingField` in parsing** — today required-field errors mostly funnel into `InvalidFormat`; distinct variants would improve diagnostics.
11. **Reconcile `list --tag` mode semantics** — either implement the documented NextGen-only gate for `adrs list --tag`, or update CLI help and OBS-LIST-003 to say the filter is available whenever parsed ADRs contain tags.

---

## 9. Assumption Inventory

> Every reverse-engineered specification rests on assumptions about *method*, *scope*, and *world*. The Fool / Socratic pass made them explicit. Each assumption is rated by confidence and tagged by where it bites if wrong.
>
> Legend: **Stated** = called out in the original spec body; **Unstated** = was implicit until this section was written.
> Confidence: **H** = directly evidenced in code, **M** = inferred from one or two code sites, **L** = plausible but not verified.

### 9.1 Methodological Assumptions (about how the spec was produced)

| # | Assumption | Stated? | Confidence | Where it bites if wrong |
|---|---|---|---|---|
| A-M-01 | Static code reading is sufficient to characterise runtime behaviour. No tests were executed and no binary was run. | Unstated | M | Conditional compilation (`mcp` / `mcp-http` features), `time` `local-offset` quirks, and dependency upgrades can diverge from the source view. |
| A-M-02 | The subagent's secondary report (line numbers, symbol names, "line 420+") is correct. The author of this spec did not independently verify every cited range. | Unstated | M | Citations may drift; reviewers using line-number anchors may miss the actual code. |
| A-M-03 | `Cargo.toml` line ranges quoted in §2.1 reflect the file as it stands. They were inferred, not re-read line-by-line. | Unstated | L | Doc-link rot if the file is reformatted. |
| A-M-04 | The workspace version `0.7.3` (in `Cargo.toml`) is the version that "ships." The published crate version on crates.io and the latest git tag may differ. | Stated | M | Spec may describe an unreleased or historical revision. |
| A-M-05 | Tests in `tests/` actually pass on the inspected commit; their existence was used as evidence of behaviour. | Unstated | L | A test file does not prove the assertions inside it currently pass. |
| A-M-06 | EARS-form "shall" statements are *descriptive* observations, not *prescriptive* requirements ratified by maintainers. | Now stated (callout box) | H | Misreading "shall" as a contract could freeze incidental behaviour as a feature. |

### 9.2 Behavioural Assumptions (about what the code does)

| # | Assumption | Stated? | Confidence | Where it bites if wrong |
|---|---|---|---|---|
| A-B-01 | Mode (`Compatible` / `NextGen`) controls *writes only*; both modes can *read* either format. (§5.1) | Stated | M | If a NextGen-mode `status` rewrite of a Compatible file converts it to YAML, the "read-symmetry" claim is broken. |
| A-B-02 | `next_number = max(existing) + 1`. The system never reuses gaps. | Stated (OBS-NEW-001) | M | If `Repository::list()` skips a parse-broken file, `next_number` may collide with an existing number on disk. |
| A-B-03 | `Repository::list()` silently drops files that fail to parse, but OBS-LIST-001 promises a "sorted listing of ADRs." | Partially stated (only in §10/non-functional notes from the report) | M | Users can have ADRs that exist on disk but are invisible to `list`, `search`, `doctor`. Need to reconcile with `list_with_errors`. |
| A-B-04 | `init` *preserves* existing ADRs (OBS-INIT-003), yet `Error::AdrDirExists(path)` is defined. The two cannot both be unconditionally true. | Stated as conflict in §7 | L | One of OBS-INIT-003 or §5.5 is wrong; consumers may rely on the wrong one. |
| A-B-05 | Status parsing is effectively infallible because `AdrStatus::Custom(String)` is a catch-all. | Stated in §7 | M | If a future tightening rejects unknown statuses, downstream tooling will break. |
| A-B-06 | `--supersedes N` and `link` writes are *atomic from the user's view* — both files updated successfully or none. The spec describes the happy-path order but no rollback. | Unstated | L | A crash between writing the new ADR and updating the old one leaves the repository in a half-updated state. |
| A-B-07 | The fuzzy-match "2× top-vs-runner-up" disambiguation rule is the actual heuristic. (OBS-EDIT-002.) | Stated | M | This was reported by the subagent at one location; not independently re-read. |
| A-B-08 | Slug rules hold for *all* inputs because property tests cover them. Empty titles, all-non-ASCII titles, titles that slug to the empty string are not separately verified in this spec. | Stated as proptest evidence | M | A title like `"日本語"` may produce an empty slug → filename collision. |
| A-B-09 | `Compatible` mode never emits YAML frontmatter — even on `update_metadata` of a file that already has frontmatter. | Unstated | L | If true, "round-trip" through Compatible mode would silently strip frontmatter. |
| A-B-10 | The MCP `update_tags`, `bulk_update_status`, `compare_adrs`, `suggest_tags` tools have the parameter shapes summarised in §4.11. The schemas were paraphrased from line-number references, not extracted from the JSON Schema emitted by `tower-mcp`. | Unstated | L | Clients writing against this spec may submit malformed payloads. |

### 9.3 Domain & Environment Assumptions (about the world the tool runs in)

| # | Assumption | Stated? | Confidence | Where it bites if wrong |
|---|---|---|---|---|
| A-D-01 | A single user invokes `adrs` at a time; no concurrent writers. | Implicit (called out in §5.2 as a *gap*, but the spec body assumes it) | M | CI parallelism, MCP HTTP multi-tenant use, or two engineers running `new` simultaneously will collide. |
| A-D-02 | The filesystem is local, POSIX-flavoured, case-sensitive, and reliable. | Unstated | M | macOS HFS+/APFS case-insensitivity may cause `0001-Foo.md` vs `0001-foo.md` collisions; network filesystems may break atomicity assumptions. |
| A-D-03 | ADR files are UTF-8 encoded. | Unstated | H | Non-UTF-8 files (e.g., from Windows editors writing UTF-16 BOM) will fail to parse with `Io` errors that the spec doesn't enumerate per-command. |
| A-D-04 | `$EDITOR` / `$VISUAL` resolution and platform fallbacks are correctly handled by the `edit` crate. | Unstated | M | Bugs in that crate (e.g., quoting, Windows `notepad` vs `notepad.exe`) propagate untested through `new`/`edit`. |
| A-D-05 | All ADRs fit comfortably in memory; `Repository::list()` reads them all eagerly (§5.3). | Stated | H | Repositories with thousands of ADRs may make `list`, `search`, MCP `list_adrs` slow. |
| A-D-06 | Filesystem walking is bounded to `max_depth = 1` — sub-folders inside `doc/adr/` are ignored. | Stated | M | Users who organise ADRs by subteam (`doc/adr/team-a/0001-…md`) silently lose them. |
| A-D-07 | `time` 0.3 with `local-offset` is invoked from a single-threaded context (a known soundness requirement of the crate). | Unstated | L | If a future change calls it after spawning threads, it may panic or return UTC. |
| A-D-08 | The MCP server's stdio transport is being driven by exactly one trusted client (Claude Desktop or equivalent). | Unstated | M | A multiplexed stdio host could interleave write tool calls, hitting A-D-01. |
| A-D-09 | `whoami::username()` returns a sensible non-empty string (used as the default `--author` for `generate book`). | Stated | H | Containers without `/etc/passwd` may yield "Unknown" or empty author. |
| A-D-10 | No git integration is wanted or expected; users version ADRs externally. | Stated in §7 as an open question | L | If maintainers want auto-commit, `update_metadata` would need a different transactional model. |

### 9.4 Compatibility & Format Assumptions

| # | Assumption | Stated? | Confidence | Where it bites if wrong |
|---|---|---|---|---|
| A-C-01 | "Drop-in compatible with `npryce/adr-tools`" means the same *file format and config file*, not the same *CLI flag set*. (`adrs link`, `adrs status`, `adrs new` differ in flag shape from upstream.) | Partially stated | M | Users porting muscle memory from `adr` to `adrs` will be surprised; scripts that shell out are not portable. |
| A-C-02 | The JSON-ADR v1 schema in `schema/json-adr/v1.json` exactly matches the structures produced by `JsonAdr` / `JsonAdrBulkExport`. The two are not generated from one source. | Unstated | M | Schema drift could let invalid JSON pass `--pretty` export but fail external schema validators. |
| A-C-03 | The `ADR001`–`ADR017` lint rules are stable identifiers. They come from `mdbook-lint-rulesets::adr` (an external crate). | Unstated | L | A minor-version bump of that crate could re-number rules and break user CI gates. |
| A-C-04 | MADR 4.0.0 fields (`decision_makers`, `consulted`, `informed`, `decision_drivers`, `considered_options`) survive a write→read→write round trip. | Stated as Q in §7 | L | If `update_metadata` doesn't rewrite these, content is silently lost on edits. |
| A-C-05 | Custom templates are leaf templates, not extensions of built-ins. (No `{% extends %}` of `nygard` / `madr`.) | Stated as Q in §7 | M | Documentation that suggests inheritance would be incorrect. |

### 9.5 Security Assumptions

| # | Assumption | Stated? | Confidence | Where it bites if wrong |
|---|---|---|---|---|
| A-S-01 | The slug pipeline strips *all* dangerous path characters; titles cannot produce filenames containing `/`, `..`, or `\0`. | Stated | M | Verified for the canonical Latin code path; Unicode classes (e.g., RTL marks, full-width slashes) not separately audited. |
| A-S-02 | The `edit` crate spawns the editor without shell interpolation, so a hostile `$EDITOR` value cannot escape into a shell. | Stated | L | Not independently verified; relies entirely on the upstream crate's posture. |
| A-S-03 | YAML frontmatter is parsed with `serde_yaml` 0.9 strict struct deserialisation, so YAML's "billion laughs" / alias-bomb attacks are bounded. | Stated | M | `serde_yaml` 0.9 has known DoS-class issues for adversarial input; not analysed against ADR threat model. |
| A-S-04 | The MCP HTTP transport is reachable only from `localhost` (or a trusted LAN). The spec offers no auth and the README does not warn. | Stated as gap | M | If a user binds it to `0.0.0.0`, anyone on the network can call `create_adr` / `update_status`. |
| A-S-05 | `import` from `--dir <PATH>` does not follow symlinks outside the directory. | Unstated | L | A malicious bundle with relative path symlinks could read or overwrite arbitrary files. |

### 9.6 Probing Questions (for maintainers)

Grouped by the theme they probe.

**Spec-as-contract**
- If a maintainer fixes a bug in (say) `Repository::list`'s silent-skip behaviour, does that violate this spec? If yes, is that intentional?
- Which OBS- requirements are *normative* (downstream tools depend on them) and which are *incidental* (could change without notice)?

**adr-tools compatibility**
- Is "compatible" defined as round-trip file format only, or also CLI flag-for-flag?
- What `npryce/adr-tools` features (e.g., `index`, alternative numbering schemes, link directionality) are *intentionally* not implemented?

**Concurrency & atomicity**
- Are `new`, `link`, and `supersede` expected to be safe under concurrent invocation? Is it acceptable to require a single-writer environment, or should we add file locks?
- If a process is killed between writing the new ADR and updating the superseded one, what's the recovery story?

**Mode interaction**
- Does an `adrs status 3 superseded` invocation in Compatible mode against a file that already has YAML frontmatter preserve the frontmatter or strip it?
- Is `--ng` "sticky" once set in `adrs.toml`, or can a single-invocation `--no-ng` (does not exist today) downgrade?

**MCP boundary**
- Should the HTTP transport ship with auth before being recommended in the README?
- Is per-call `Repository::open` intentional for consistency, or a performance accident?

**Encoding & i18n**
- What is the slug for `"日本語"`? For `""`? For `"---"`? Are any of those a security concern?
- Are non-UTF-8 ADR files an error or a silent skip?

### 9.7 Suggested Experiments

| Assumption to test | Experiment | Effort | Signal |
|---|---|---|---|
| A-M-01 (static-only) | Run `cargo test --workspace` and re-check whether OBS-* claims pass. | Low | Test failures localise wrong observations. |
| A-B-02 / A-B-03 (numbering vs silent skip) | Drop a malformed `0042-broken.md` into the corpus, run `adrs new`. | Low | If it produces another `0042-…md`, A-B-02 is violated. |
| A-B-04 (`init` over non-empty dir) | Run `adrs init` in a populated dir; capture stdout/exit. | Low | Either OBS-INIT-003 stands or `AdrDirExists` fires. |
| A-B-08 (slug edge cases) | Property-test `slug("")`, `slug("///")`, `slug("日本語")`. | Low | Empty/duplicate filenames are a real bug. |
| A-B-09 (Compatible-mode rewrite of FM file) | Create an ADR with frontmatter, switch to Compatible mode, run `adrs status N accepted`. | Low | Diff the file; FM-strip is the failure mode. |
| A-D-01 (concurrency) | Run two `adrs new "X"` invocations in parallel under `parallel`/`xargs`. | Low | Two files with the same number = collision confirmed. |
| A-D-06 (depth=1) | Create `doc/adr/sub/0099-x.md`; run `adrs list`. | Low | If hidden, document the limitation. |
| A-C-02 (schema vs structs) | `cargo run -- export json --pretty | ajv validate -s schema/json-adr/v1.json`. | Med | Schema drift surfaces immediately. |
| A-C-04 (MADR round trip) | Create MADR ADR with all optional fields; run `adrs status` then re-read; diff. | Med | Field loss = bug + spec error. |
| A-S-04 (MCP HTTP exposure) | Bind `--http 0.0.0.0:3000`; from another host call `create_adr`. | Low | Confirms the unauthenticated-write risk. |

---

## 10. Static Verification Pass (2026-04-25)

### 10.1 Purpose and Scope

This pass verifies the features described in this reverse-engineered spec against the workspace code as it exists on 2026-04-25. It is intentionally **static-only**: no `cargo test`, CLI command execution, JSON Schema validation, or temporary repository experiments were run.

User value: this section turns the spec from a first-pass code reading into an auditable map of which feature claims are confirmed, which claims need correction, and which areas remain runtime-only questions.

### 10.2 Verification Summary

| Result | OBS IDs / areas | Evidence summary |
|---|---|---|
| Confirmed | CFG-002..006, INIT-001..003, NEW-001..008, EDIT-001..002, LIST-001..002 and LIST-004, SRCH-001..004, LINK-001, STAT-001..003, CONF-001, DOC-001..003, GEN-001..003, EXP-001..004, IMP-001..004, TMPL-001..004, MCP-002..005, PARSE-001..004 | Implementations in `crates/adrs-core/src/*.rs`, `crates/adrs/src/main.rs`, `crates/adrs/src/commands/*.rs`, and `crates/adrs/src/mcp.rs` match the described behaviour under static inspection. |
| Needs wording clarification | CFG-001 | `discover()` checks `$ADRS_CONFIG`, then `search_upward()`. Inside `search_upward()`, each directory level checks `adrs.toml`, then `.adr-dir`, then `doc/adr`, before moving upward. |
| Contradicted | LIST-003 | `commands::list` has no NextGen-mode validation for `--tag`; it applies `adr.tags` filtering unconditionally. |
| Needs command wording correction | MCP-001 | The server is started by `adrs mcp serve`; `--http` is an option on the `serve` subcommand when the `mcp-http` feature is enabled. |

### 10.3 Functional Verification Notes

- **Configuration discovery**: `$ADRS_CONFIG`, `$ADR_DIRECTORY`, project config, global config, and defaults are all implemented. The project-config search should be described as nearest project marker with same-directory `adrs.toml` priority, not as a global search for the nearest `adrs.toml` before considering any `.adr-dir`.
- **Creation and mutation flows**: `Repository::new_adr`, `supersede`, `link`, `set_status`, and `update_metadata` support the OBS-NEW, OBS-LINK, and OBS-STAT claims. These operations are not transactional across multiple files.
- **Tag semantics**: `adrs new --tags` and MCP `update_tags` correctly reject Compatible mode. `adrs list --tag` does not reject Compatible mode, despite the CLI help and OBS-LIST-003 saying it requires NextGen.
- **Parsing**: frontmatter detection, legacy H2 extraction, `superceded` typo tolerance, and scalar-or-array YAML fields are implemented. `Repository::list()` still silently drops parse failures, while `check_all()` uses `list_with_errors()` to report them to `doctor`.
- **MADR fidelity**: template generation includes MADR sections, and frontmatter supports `decision-makers`, `consulted`, and `informed`. `decision_drivers`, `considered_options`, `confirmation`, and `custom_sections` are not represented in `Adr`; JSON export currently emits those fields empty or absent.
- **Frontmatter preservation**: `update_metadata()` preserves body text and unknown frontmatter keys for status/link/tag rewrites by editing only selected YAML blocks. Full rewrites through `Repository::update()` render from the typed `Adr` model and are not a general unknown-field preservation mechanism.
- **MCP HTTP security**: the spec's unauthenticated HTTP gap is confirmed. `serve_http()` constructs `HttpTransport::new(router)` directly, with no auth middleware in this codebase.

### 10.4 Acceptance Criteria Verification

| Acceptance criterion | Static verdict | Notes |
|---|---|---|
| AC-001 | Supported by code | `Repository::init()` creates `doc/adr`, `.adr-dir`, and bootstrap ADR in Compatible mode. |
| AC-002 | Supported by code | `Repository::init(..., ng = true)` writes `adrs.toml` and sets NextGen mode. |
| AC-003 | Supported by code | `new_adr()` uses `next_number()` and default `Proposed` status. |
| AC-004 | Supported by code, non-atomic | `supersede()` writes the new ADR, then updates the old ADR. A crash between writes can leave partial state. |
| AC-005 | Supported by code | `new --tags` bails unless `--ng` or config mode is NextGen. |
| AC-006 | Supported by code | `find()` returns `AmbiguousAdr` unless the top fuzzy score is more than 2x the runner-up. |
| AC-007 | Supported by code | Status filters are conjunctive and long output includes number, status, date, title. |
| AC-008 | Partially supported by static evidence | `doctor` runs collection rules and exits on errors; exact duplicate rule ID is delegated to `mdbook-lint-rulesets::adr`. |
| AC-009 | Partially supported by static evidence | Export/import paths exist; round-trip quality was not executed or schema-validated in this pass. |
| AC-010 | Supported by code | `generate_graph()` emits `digraph`, dotted sequence edges, and labelled link edges. |
| AC-011 | Supported by code | MCP `update_tags_impl()` errors when `repo.config().is_next_gen()` is false. |
| AC-012 | Supported by code | Custom templates load only when no CLI `--format` or `--variant` override is present. |
| AC-013 | Supported for listed command | `main.rs` uses the discovered root for `list`; this pass did not verify every import/export `--cwd` interaction. |
| AC-014 | Current implementation behaviour | Given a Compatible-mode repository with parsed ADR tags, when the user runs `adrs list --tag <TAG>`, then the system applies the tag filter and does not error solely because of Compatible mode. |

### 10.5 Error Handling Verification

| Case | Static verdict |
|---|---|
| `new --tags` in Compatible mode | Returns an `anyhow::bail!` error before tag metadata is written. |
| `list --tag` in Compatible mode | No mode error is implemented; this contradicts OBS-LIST-003. |
| MCP `update_tags` in Compatible mode | Returns an MCP tool error string and does not update tags. |
| `status --by` with non-`superseded` status | Returns a validation error before repository mutation. |
| `doctor` warnings only | Prints warnings and returns success; only Error-severity issues call `std::process::exit(1)`. |
| Malformed ADR files in `doctor` | `check_all()` records parse errors as Error-severity issues via `list_with_errors()`. |

### 10.6 Implementation TODO Checklist

- [ ] Decide whether `adrs list --tag` should be NextGen-only or mode-agnostic.
- [ ] If NextGen-only, pass discovered config or `is_next_gen` into `commands::list()` and reject `tag.is_some()` in Compatible mode.
- [ ] If mode-agnostic, update OBS-LIST-003 and CLI help text to remove the NextGen-only claim.
- [ ] Add CLI tests for `list --tag` in Compatible and NextGen repositories.
- [ ] Tighten OBS-CFG-001 wording to describe level-wise project config discovery.
- [ ] Replace `adrs mcp` wording with `adrs mcp serve` in OBS-MCP-001 and related docs.
- [ ] Run `cargo test --workspace` to upgrade this section from static verification to executed verification.
- [ ] Add targeted experiments for the remaining runtime assumptions in §9.7, especially concurrent `new`, slug edge cases, schema-vs-struct validation, and MADR round-trip fidelity.

# Requirements Specification: `pirs`

> **Source inputs**: [`spec/adrs_reverse_spec.md`](adrs_reverse_spec.md), post-major incident review guidance supplied by the stakeholder, and product requirements from the 2026-04-25 planning session.
> **Method**: Requirements adaptation in the style of the `adrs` reverse specification. This document is not reverse-engineered from an existing `pirs` implementation.
> **Format**: Proposed requirements expressed in [EARS](https://alistairmavin.com/ears/) syntax with Given/When/Then acceptance criteria.

> **Read this first - draft vs implementation.**
> Every "shall" in this document is a proposed requirement for a new tool named `pirs`, not an observation of code that exists today. Treat this as a build target and validation checklist for implementation.

---

## 1. Overview

`pirs` is a CLI and optional Model Context Protocol (MCP) server for creating and managing **Post-Incident Reviews (PIRs)**.

The tool is intended for both humans and LLM agents. It should make incident logging lightweight enough that an agent can record small development incidents, such as failing tests or broken builds, while still supporting larger operational incidents, such as production outages, security events, or process failures.

The primary value of `pirs` is to turn incidents into structured learning records with clear ownership and follow-up. A PIR must capture what happened, who or what was involved, how the incident unfolded, how long it took to discover and resolve, the 5 Whys analysis, and the actions required to prevent recurrence.

`pirs` is designed to be:

- **Agent-friendly**: safe for non-interactive use by LLMs and automation during code changes, tests, deployments, and incident response.
- **Human-readable**: stores PIRs as Markdown with structured YAML frontmatter.
- **Machine-readable**: supports JSON export/import and MCP tools so agents can query, create, and update reviews.
- **Blameless by default**: records people, agents, systems, and teams involved without assigning personal blame.
- **Action-oriented**: treats follow-up actions as first-class entities with owners, due dates, statuses, and evidence.

---

## 2. Product Goals

| Goal | Description |
|---|---|
| Capture incidents early | Make it cheap to record a test failure, broken build, bad migration, outage, or security concern as soon as it appears. |
| Preserve context | Store command output snippets, timeline events, actor metadata, links, and decision points before they are lost. |
| Support agent-only incidents | Allow an LLM agent to be the sole actor when it detects and fixes a local development incident. |
| Standardize PIR structure | Ensure every review has a problem statement, timeline, discovery and resolution timing, 5 Whys, and actions. |
| Enable continuous improvement | Provide searchable history and aggregate action/incident reports. |
| Keep records versionable | Use plain files that work naturally with git and code review. |

---

## 3. Architecture Summary

### 3.1 Proposed Technology Stack

`pirs` should follow the implementation shape of `adrs` unless there is a PIR-specific reason to diverge.

| Concern | Proposed Choice | Rationale |
|---|---|---|
| Language / edition | Rust, edition 2024 | Matches `adrs`; fast startup and reliable single-binary distribution. |
| CLI parsing | `clap` derive | Consistent subcommand UX with `adrs`. |
| Errors | `thiserror` in core, `anyhow` in bin | Library-first error model with ergonomic CLI reporting. |
| Serialization | `serde`, `serde_yaml`, `toml`, `serde_json` | YAML frontmatter, config, and JSON-PIR export/import. |
| Markdown | `pulldown-cmark` or equivalent | Parse and preserve Markdown sections. |
| Templates | `minijinja` | Built-in and custom PIR templates. |
| Filesystem walk | `walkdir` | Repository scans bounded to PIR directory. |
| Date / time | `time` or `chrono` | Timeline events and duration calculation. |
| Fuzzy search | `fuzzy-matcher` | Lookup by number, title, service, or problem statement. |
| Editor spawn | `edit` | Human edit workflow. |
| MCP server | `tower-mcp` + `tokio` | Direct LLM integration, mirroring `adrs`. |
| Testing | `assert_cmd`, `assert_fs`, `predicates`, `tempfile`, `proptest` | CLI and core behaviour coverage. |

### 3.2 Proposed Workspace Layout

```text
pirs/
|-- crates/
|   |-- pirs/              # CLI binary + MCP server
|   |   |-- src/
|   |   |   |-- main.rs    # clap dispatch, global flags, --cwd
|   |   |   |-- mcp.rs     # MCP server (stdio + optional HTTP)
|   |   |   |-- commands/  # one file per subcommand
|   |   |-- tests/         # CLI integration tests
|   |-- pirs-core/         # Library: all PIR business logic
|       |-- src/
|       |   |-- config.rs      # discovery, pirs.toml / .pir-dir
|       |   |-- error.rs       # thiserror error enum
|       |   |-- export.rs      # JSON-PIR v1 export/import
|       |   |-- lib.rs         # public API surface
|       |   |-- lint.rs        # PIR validation rules
|       |   |-- parse.rs       # YAML frontmatter + markdown parser
|       |   |-- repository.rs  # CRUD over PIR collection
|       |   |-- template.rs    # built-in and custom templates
|       |   |-- types.rs       # Pir, IncidentStatus, TimelineEvent, ActionItem
|-- schema/json-pir/v1.json
|-- doc/pir/                   # default PIR repository
|-- book/                      # optional mdBook documentation
|-- tests/fixtures/pir-corpus/
```

### 3.3 Data Flow

```text
       CLI args --> clap --> subcommand handler --+
                                                   |
                                                   v
                              Config::discover(cwd) --> pirs.toml | .pir-dir | $PIRS_CONFIG | defaults
                                                   |
                                                   v
                                       Repository::open(root)
                                                   |
                          +------------------------+-------------------------+
                          v                        v                         v
                       Parser                  walkdir                   Template
                (YAML frontmatter + MD)        (NNNN-*.md)              (minijinja)
                          |                        |                         |
                          +------------------------+-------------------------+
                                                   |
                                                   v
                         stdout / stderr / file writes / JSON output / exit code

MCP path:
       LLM client --> MCP transport --> PirState{root, actor} --> Repository::open(root) per call
```

---

## 4. Core PIR Data Model

### 4.1 PIR File Naming

PIR files should use stable numeric identifiers and slugs:

```text
doc/pir/0001-failing-auth-tests-after-token-change.md
doc/pir/0002-production-api-500s-during-deploy.md
```

### 4.2 Required Fields

Every PIR shall support these fields as first-class structured data.

| Field | Required | Description |
|---|---:|---|
| `number` | Yes | Sequential PIR number. |
| `title` | Yes | Short human-readable incident title. |
| `problem_statement` | Yes | Clear statement of the observed failure, impact, and expected behaviour. |
| `status` | Yes | `Open`, `Investigating`, `Mitigated`, `Resolved`, `Reviewed`, or `Cancelled`. |
| `severity` | Yes | `Low`, `Medium`, `High`, `Critical`, or custom value. |
| `incident_type` | Yes | `Development`, `Production`, `Security`, `Process`, or custom value. |
| `detected_at` | Yes | When the incident was discovered. |
| `occurred_at` | Recommended | Best known start time of the incident. |
| `resolved_at` | Required before `Resolved` | When service, tests, or workflow returned to acceptable state. |
| `time_to_discover` | Derived | `detected_at - occurred_at` when both timestamps exist. |
| `time_to_resolve` | Derived | `resolved_at - detected_at` when both timestamps exist. |
| `people_involved` | Yes | Humans, teams, systems, or agents involved. Agent-only is valid. |
| `timeline` | Yes | Ordered timeline events with timestamp, actor, event type, and description. |
| `five_whys` | Required before `Reviewed` | Ordered causal analysis entries. |
| `actions` | Required before `Reviewed` | Follow-up action items with owner, due date, status, and evidence. |

### 4.3 Recommended Fields

| Field | Description |
|---|---|
| `impact` | Systems, users, tests, environments, or workflows affected. |
| `summary` | Short executive summary after review. |
| `detection_method` | How the incident was discovered: test, alert, user report, agent observation, monitoring, audit, etc. |
| `root_cause` | Concise root cause once known. |
| `contributing_factors` | Non-root factors that made the incident more likely or worse. |
| `what_went_well` | Positive response behaviours to preserve. |
| `what_went_wrong` | Process or technical gaps to improve. |
| `where_we_got_lucky` | Risks that did not materialize but could have. |
| `links` | Related commits, PRs, issues, ADRs, logs, dashboards, deployments, runbooks. |
| `tags` | Search and grouping labels. |
| `confidentiality` | `Public`, `Internal`, `Restricted`, or custom value. |
| `agent_context` | Model/session/tool metadata when an LLM created or updated the PIR. |

### 4.4 Example PIR Frontmatter

```yaml
---
number: 12
title: Failing payment tests after checkout refactor
status: Reviewed
severity: Medium
incident_type: Development
problem_statement: >-
  The checkout test suite failed after a refactor because payment retry state was not preserved
  across the new service boundary.
occurred_at: 2026-04-25T10:04:00Z
detected_at: 2026-04-25T10:06:12Z
resolved_at: 2026-04-25T10:43:30Z
time_to_discover: PT2M12S
time_to_resolve: PT37M18S
detection_method: agent-test-run
people_involved:
  - name: GitHub Copilot
    type: agent
    role: implementer
timeline:
  - at: 2026-04-25T10:06:12Z
    actor: GitHub Copilot
    type: detected
    description: cargo test failed in checkout retry tests
five_whys:
  - question: Why did checkout retry tests fail?
    answer: The refactor dropped retry state before payment confirmation.
actions:
  - id: ACT-001
    description: Add regression coverage for retry state persistence.
    owner: GitHub Copilot
    owner_type: agent
    due: 2026-04-25
    status: Done
tags: [tests, checkout, agent]
---
```

---

## 5. Proposed Functional Requirements (EARS)

Identifiers follow `REQ-<AREA>-<NNN>`.

### 5.1 Configuration & Discovery

**REQ-CFG-001 - Configuration discovery order**
When the CLI starts, the system shall locate configuration in the following order: (1) `$PIRS_CONFIG` if set, (2) nearest `pirs.toml` walking upward from `--cwd` or current directory, (3) nearest `.pir-dir`, (4) `~/.config/pirs/config.toml`, (5) built-in defaults.

**REQ-CFG-002 - Default PIR directory**
When no configuration is found, the system shall use `doc/pir` as the default PIR directory.

**REQ-CFG-003 - Environment directory override**
Where `$PIR_DIRECTORY` is set, the system shall use it as the effective PIR directory, overriding config file values.

**REQ-CFG-004 - Working-directory override**
When `-C <DIR>` or `--cwd <DIR>` is supplied, the system shall perform configuration discovery and filesystem operations relative to `<DIR>`.

**REQ-CFG-005 - Config dump**
When `pirs config` runs, the system shall print project root, config source, PIR directory, resolved PIR directory, template settings, privacy settings, and MCP settings.

### 5.2 `init`

**REQ-INIT-001 - Directory creation**
When `pirs init [DIRECTORY]` runs and the target directory does not exist, the system shall create it.

**REQ-INIT-002 - Config artifact**
When `pirs init` runs, the system shall create `pirs.toml` unless `.pir-dir` compatibility output is explicitly requested.

**REQ-INIT-003 - No fake incident by default**
When `pirs init` runs against an empty PIR directory, the system shall not create a sample PIR unless `--sample` is supplied.

**REQ-INIT-004 - Non-empty preservation**
When `pirs init` runs against a directory that already contains PIRs, the system shall preserve them and report the existing count.

### 5.3 `new` / Incident Creation

**REQ-NEW-001 - Sequential numbering**
When `pirs new "<TITLE>"` runs, the system shall assign `next_number = max(existing) + 1` or `1` if none exist.

**REQ-NEW-002 - Slug rules**
The system shall produce slugs that are lowercase, contain only `[a-z0-9-]`, never start or end with `-`, and never contain consecutive `-`.

**REQ-NEW-003 - Problem statement capture**
When `pirs new` runs, the system shall require a problem statement via `--problem`, `--from-file`, interactive prompt, or editor completion before the PIR can be marked `Reviewed`.

**REQ-NEW-004 - Agent-only participant support**
When `--agent <NAME>` is supplied and no human participant is supplied, the system shall create a valid PIR whose `people_involved` list contains the agent as an actor.

**REQ-NEW-005 - Incident classification**
When `--type`, `--severity`, `--service`, `--environment`, or `--tag` are supplied, the system shall store them as structured metadata.

**REQ-NEW-006 - Initial timeline event**
When a PIR is created, the system shall add an initial timeline event of type `detected` unless `--no-initial-event` is supplied.

**REQ-NEW-007 - Non-interactive mode**
While stdin is not a TTY or `--no-edit` is supplied, when required creation fields are missing, the system shall fail with a clear validation error instead of opening an editor.

**REQ-NEW-008 - Editor invocation**
While `--no-edit` is not set and an interactive terminal is available, when the new PIR file has been written, the system shall open it in `$EDITOR` or platform fallback.

### 5.4 `run` / Agent Workflow Logging

**REQ-RUN-001 - Command wrapper**
When `pirs run -- <COMMAND>` is invoked, the system shall execute `<COMMAND>` and return the same exit code unless `--pirs-exit-code` overrides that behaviour.

**REQ-RUN-002 - Failure PIR creation**
When `pirs run --on-fail create -- <COMMAND>` observes a non-zero exit code, the system shall create a Development-type PIR with command, exit code, start time, finish time, captured output summary, and agent or user actor metadata.

**REQ-RUN-003 - Failure event append**
When `pirs run --on-fail append --pir <N> -- <COMMAND>` observes a non-zero exit code, the system shall append a timeline event to PIR `N` instead of creating a new PIR.

**REQ-RUN-004 - Output capture limits**
When command output is captured, the system shall enforce configurable byte limits and redact configured secret patterns before writing to disk.

**REQ-RUN-005 - Successful command handling**
When a wrapped command succeeds, the system shall not create a PIR unless `--always-log` is supplied.

### 5.5 Timeline Management

**REQ-TIME-001 - Timeline add**
When `pirs timeline add <PIR> --at <TIME> --actor <ACTOR> --type <TYPE> --message <TEXT>` runs, the system shall append an ordered timeline event to the PIR.

**REQ-TIME-002 - Timeline ordering**
When a PIR is parsed or written, the system shall preserve event timestamps and present timeline events in ascending timestamp order by default.

**REQ-TIME-003 - Duration derivation**
When `occurred_at`, `detected_at`, or `resolved_at` values change, the system shall recompute `time_to_discover`, `time_to_resolve`, and total duration where enough timestamps exist.

**REQ-TIME-004 - Manual duration override**
Where exact timestamps are unknown, the system shall allow explicit `time_to_discover` and `time_to_resolve` values with a note explaining the estimate.

### 5.6 5 Whys Analysis

**REQ-WHY-001 - Ordered 5 Whys entries**
When `pirs why add <PIR> --question <TEXT> --answer <TEXT>` runs, the system shall append an ordered 5 Whys entry.

**REQ-WHY-002 - Minimum review quality**
When a PIR is moved to `Reviewed`, the system shall require at least one 5 Whys entry and shall warn when fewer than five entries exist unless `--allow-short-analysis` is supplied.

**REQ-WHY-003 - Root cause summary**
When the final 5 Whys entry identifies a root cause, the system shall allow it to be promoted to the PIR `root_cause` field.

### 5.7 Action Management

**REQ-ACT-001 - Action creation**
When `pirs action add <PIR> --description <TEXT> --owner <OWNER> --due <DATE>` runs, the system shall add an action item with a stable action ID.

**REQ-ACT-002 - Owner types**
When an action owner is supplied, the system shall support owner types `human`, `agent`, `team`, and `system`.

**REQ-ACT-003 - Action statuses**
The system shall support action statuses `Open`, `InProgress`, `Blocked`, `Done`, `Cancelled`, and custom values.

**REQ-ACT-004 - Action completion evidence**
When `pirs action close <PIR> <ACTION_ID>` runs, the system shall allow evidence links or notes to be recorded with the completion.

**REQ-ACT-005 - Open action reporting**
When `pirs actions list` runs, the system shall list action items across all PIRs and support filters for owner, due date, status, severity, type, and tag.

### 5.8 Status and Review Workflow

**REQ-STAT-001 - Status change**
When `pirs status <PIR> <STATUS>` runs, the system shall update the PIR status while preserving body content.

**REQ-STAT-002 - Resolve requirements**
When setting status to `Resolved`, the system shall require `resolved_at` or shall set it to the current time if `--now` is supplied.

**REQ-STAT-003 - Review requirements**
When setting status to `Reviewed`, the system shall validate required fields, timeline, 5 Whys, and action items before persisting the status change.

**REQ-STAT-004 - Cancelled incidents**
When setting status to `Cancelled`, the system shall require a cancellation reason.

### 5.9 People and Actor Management

**REQ-PPL-001 - Add actor**
When `pirs people add <PIR> --name <NAME> --type <TYPE> --role <ROLE>` runs, the system shall add the actor to `people_involved` if absent.

**REQ-PPL-002 - Agent metadata**
Where an actor has type `agent`, the system shall support optional fields for model, provider, session ID, tool name, and automation context.

**REQ-PPL-003 - Blameless wording**
The system shall label the section `People and systems involved` or equivalent, and shall not require fields named `owner_of_failure`, `blame`, or `culprit`.

### 5.10 Links and Evidence

**REQ-LINK-001 - Evidence links**
When `pirs link <PIR> <URI> --kind <KIND>` runs, the system shall attach a typed link to the PIR.

**REQ-LINK-002 - Supported link kinds**
The system shall support link kinds `Commit`, `PullRequest`, `Issue`, `Log`, `Dashboard`, `Runbook`, `Deployment`, `TestRun`, `ADR`, `PIR`, and custom values.

**REQ-LINK-003 - Related PIRs**
When a PIR links to another PIR, the system shall support relationship kinds `CausedBy`, `RelatedTo`, `DuplicateOf`, `FollowUpTo`, and custom values.

### 5.11 `list`, `search`, and `show`

**REQ-LIST-001 - Sorted listing**
When `pirs list` runs, the system shall return PIRs sorted by ascending number by default.

**REQ-LIST-002 - Filters**
When any of `--status`, `--severity`, `--type`, `--actor`, `--service`, `--environment`, `--tag`, `--since`, `--until`, or `--has-open-actions` are supplied, the system shall apply them as conjunctive filters.

**REQ-LIST-003 - Output formats**
Where `--long` is supplied, the system shall print number, status, severity, detected date, time to resolve, open action count, and title per line; otherwise it shall print file paths.

**REQ-SHOW-001 - PIR display**
When `pirs show <QUERY>` runs, the system shall find a PIR by number or fuzzy title/problem statement and print a human-readable summary.

**REQ-SRCH-001 - Search scope**
When `pirs search <QUERY>` runs without filters, the system shall search title, problem statement, impact, timeline, 5 Whys, root cause, actions, and links.

**REQ-SRCH-002 - Case sensitivity**
While `--case-sensitive` is not set, the system shall perform case-insensitive matching.

### 5.12 Validation and Doctor

**REQ-DOC-001 - PIR validation**
When `pirs doctor` runs, the system shall validate required fields, duplicate numbers, malformed timestamps, impossible durations, broken links, missing action owners, overdue actions, missing 5 Whys, and unsafe secret patterns.

**REQ-DOC-002 - Severity levels**
The validation system shall classify findings as `Error`, `Warning`, or `Info`.

**REQ-DOC-003 - Exit code**
When at least one Error-severity issue is reported, the system shall exit with code `1`; warnings and info shall not affect the exit code unless `--warnings-as-errors` is supplied.

**REQ-DOC-004 - Review gate**
When `pirs doctor --review-gate <PIR>` runs, the system shall fail if that PIR is not ready to be marked `Reviewed`.

### 5.13 Reports and Metrics

**REQ-RPT-001 - PIR report generation**
When `pirs generate report <PIR>` runs, the system shall produce a Markdown report containing summary, problem statement, impact, timeline, timing metrics, 5 Whys, actions, and lessons learned.

**REQ-RPT-002 - Action register**
When `pirs generate actions` runs, the system shall produce a cross-PIR action register suitable for status review.

**REQ-RPT-003 - Metrics summary**
When `pirs metrics` runs, the system shall summarize incident counts, severity distribution, mean/median time to discover, mean/median time to resolve, recurring tags, and open action counts.

**REQ-RPT-004 - Blameless language audit**
When `pirs doctor --language` runs, the system shall warn about blame-oriented language patterns in PIR text.

### 5.14 Export / Import

**REQ-EXP-001 - JSON-PIR v1**
When `pirs export json` runs, the system shall emit JSON conforming to `schema/json-pir/v1.json`.

**REQ-EXP-002 - Single vs bulk export**
When a PIR number is supplied, the system shall emit a single JSON-PIR object; otherwise it shall emit a bulk export containing tool metadata, generated timestamp, repository info, and PIR array.

**REQ-EXP-003 - Redacted export**
Where `--redact` is supplied, the system shall remove or mask fields configured as sensitive before writing JSON.

**REQ-IMP-001 - JSON import**
When `pirs import json <FILE>` runs, the system shall accept a filesystem path or `-` for stdin.

**REQ-IMP-002 - Dry run**
Where `--dry-run` is supplied, the system shall report what would be imported and shall not write files.

**REQ-IMP-003 - Overwrite policy**
When a target PIR file already exists, the system shall skip it unless `--overwrite` is supplied.

### 5.15 Templates

**REQ-TMPL-001 - Built-in templates**
When `pirs template list` runs, the system shall list built-in templates for `development`, `production`, `security`, `process`, and `minimal` PIRs.

**REQ-TMPL-002 - Template show**
When `pirs template show <NAME>` runs, the system shall print the variable reference followed by the raw template body.

**REQ-TMPL-003 - Custom templates**
Where `templates.custom` is set in `pirs.toml`, the system shall load the custom template for new PIRs unless overridden by `--template`.

**REQ-TMPL-004 - Required sections**
Built-in full templates shall include sections for Problem Statement, Impact, People and Systems Involved, Timeline, Detection and Resolution Timing, 5 Whys, Actions, Lessons Learned, and Links.

### 5.16 MCP Server

**REQ-MCP-001 - Transports**
When `pirs mcp serve` runs without `--http`, the system shall serve MCP over stdio. Where HTTP support is enabled and `--http <ADDR>` is supplied, the system shall serve MCP over HTTP at the configured route.

**REQ-MCP-002 - Per-call repository**
When an MCP tool is invoked, the system shall open the repository fresh for that call unless a safe cache has been explicitly configured.

**REQ-MCP-003 - Read-only tools**
The MCP server shall expose read-only tools `list_pirs`, `get_pir`, `search_pirs`, `get_open_actions`, `get_repository_info`, `validate_pir`, `get_incident_metrics`, and `suggest_related_pirs`.

**REQ-MCP-003A - Incident metrics MCP tool**
When the `get_incident_metrics` MCP tool is called, the system shall open the repository for that call, apply optional `status`, `severity`, `incident_type`, `tag`, and `has_open_actions` filters using the same semantics as `list_pirs`, and return a stable JSON object containing the selected filter scope plus the `IncidentMetrics` fields `total`, `by_status`, `by_severity`, `by_type`, `ttd_seconds`, `ttr_seconds`, `recurring_tags`, `open_actions`, and `total_actions`. Where `include_text` is true, the system shall also include the same human-readable metrics summary used by `pirs metrics`.

**REQ-MCP-003B - Related PIR suggestion MCP tool**
When the `suggest_related_pirs` MCP tool is called with a PIR number, the system shall open the repository for that call, return an MCP error result if the target PIR does not exist, score every other PIR using deterministic local metadata and text signals, and return at most `limit` suggestions ordered by descending score and then ascending PIR number. The tool shall cap `limit` at 20, default it to 5, and omit candidates below `min_score`, defaulting `min_score` to 1. Scores shall be unsigned integers in the range 0..100 for a given scoring version, with future weight tuning allowed to change absolute scores while preserving bounded output and deterministic ordering rules.

**REQ-MCP-003C - Related PIR response privacy boundary**
The `suggest_related_pirs` MCP tool shall not return PIR body excerpts, root-cause text, timeline text, 5 Whys text, or action descriptions. Each suggestion shall include only the candidate PIR number, title, status, severity, incident type, tags, numeric score, and bounded non-secret matching signals: up to five shared tags, shared tag count, shared token count, same incident type, same severity, and explicit PIR-link presence. Shared text tokens shall be counted but not returned.

**REQ-MCP-004 - Write tools**
The MCP server shall expose write tools `create_pir`, `log_incident`, `append_timeline_event`, `update_status`, `add_why`, `add_action`, `update_action`, `link_evidence`, and `finalize_review`.

**REQ-MCP-005 - Agent attribution**
When an MCP write tool is called, the system shall record actor metadata identifying the agent or automation client when available.

**REQ-MCP-006 - HTTP authentication**
Where MCP HTTP transport is enabled, the system shall support bearer-token authentication and shall warn when binding to a non-loopback address without authentication.

### 5.17 Parsing and File Format

**REQ-PARSE-001 - Frontmatter detection**
When parsing a PIR file beginning with `---\n`, the system shall extract YAML frontmatter and parse structured metadata.

**REQ-PARSE-002 - Markdown sections**
When parsing the body, the system shall recognize H2 sections for Problem Statement, Impact, People and Systems Involved, Timeline, Detection and Resolution Timing, 5 Whys, Actions, Lessons Learned, and Links.

**REQ-PARSE-003 - Unknown key preservation**
When updating metadata surgically, the system shall preserve unknown YAML keys unless explicitly removed by the user.

**REQ-PARSE-004 - Flexible actor fields**
When deserializing `people_involved`, the system shall accept either strings or structured actor objects, normalizing strings into actor records.

---

## 6. Non-Functional Requirements

### 6.1 Agent and Automation UX

- The CLI shall support fully non-interactive operation for all creation and update workflows.
- The CLI shall keep diagnostics on stderr and machine-readable command output on stdout when `--json` is supplied.
- Commands intended for agents shall be idempotent where practical, especially appending duplicate timeline events or action items.
- Startup time should remain under 50 ms for common commands on a small repository.

### 6.2 Security and Privacy

- The system shall support redaction patterns for secrets, tokens, credentials, API keys, and private URLs before captured command output is persisted.
- The system shall support a `confidentiality` field and allow restricted PIRs to be excluded from default exports.
- The MCP HTTP transport shall not be recommended for non-local use without authentication.
- The system shall avoid shell interpolation when invoking editors or wrapped commands.
- Security-type PIRs shall support extra warnings when required evidence or containment actions are missing.

### 6.3 Reliability and Concurrency

- The system shall use file locking or atomic create semantics around `next_number()` and file creation.
- Multi-file updates shall either be transactional or leave enough recovery information for `pirs doctor` to detect partial writes.
- Timeline event append operations shall avoid data loss under concurrent agent calls.

### 6.4 Compatibility and Portability

- The system shall use portable paths and work on macOS, Linux, and Windows.
- PIR files shall be UTF-8 encoded Markdown.
- The default file format shall remain useful in git diffs and code review.

### 6.5 Performance

- Repository scans may read all PIR files into memory for ordinary team-scale repositories.
- The implementation should avoid unbounded command-output capture.
- `pirs list`, `pirs search`, and `pirs actions list` should remain responsive for at least 10,000 PIR files.

### 6.6 Blameless Review Practice

- The tool shall frame reviews around systems, decisions, contributing factors, and improvement actions.
- The tool shall record accountability for action ownership without assigning blame for the incident.
- The tool shall preserve what went well as well as what failed.

---

## 7. Acceptance Criteria

### AC-001 - Initialize a PIR repository

- **Given** an empty project with no `pirs.toml` and no `.pir-dir`
- **When** the user runs `pirs init`
- **Then** the system creates `doc/pir/`, writes `pirs.toml`, and does not create a fake incident record.

### AC-002 - Create an agent-only development incident

- **Given** an initialized repository
- **When** an LLM runs `pirs new "Failing cargo test after parser change" --type development --severity medium --agent "GitHub Copilot" --problem "cargo test failed after parser metadata update" --no-edit`
- **Then** the system writes the next numbered PIR file with the problem statement, agent participant, initial detected timeline event, and status `Open`.

### AC-003 - Log a failing command automatically

- **Given** an initialized repository
- **When** an agent runs `pirs run --on-fail create -- cargo test` and `cargo test` exits non-zero
- **Then** the system creates a Development PIR containing the command, exit code, redacted output summary, detected timestamp, and agent actor metadata, and returns the original command exit code.

### AC-004 - Append timeline and compute durations

- **Given** PIR `0003` has `occurred_at` and `detected_at`
- **When** the user runs `pirs status 3 resolved --now`
- **Then** the system sets `resolved_at`, computes `time_to_discover`, computes `time_to_resolve`, and appends a resolution timeline event.

### AC-005 - Complete 5 Whys analysis

- **Given** PIR `0003` exists
- **When** the user adds five ordered `why` entries and promotes the final answer to root cause
- **Then** `pirs show 3` displays the ordered 5 Whys and the root cause summary.

### AC-006 - Track follow-up actions

- **Given** PIR `0003` exists
- **When** the user runs `pirs action add 3 --description "Add regression test" --owner "GitHub Copilot" --owner-type agent --due 2026-04-25`
- **Then** the system adds a stable action ID and `pirs actions list --owner "GitHub Copilot"` includes it.

### AC-007 - Prevent premature review closure

- **Given** PIR `0003` has no action items and no 5 Whys entries
- **When** the user runs `pirs status 3 reviewed`
- **Then** the system exits non-zero with validation errors and leaves the PIR status unchanged.

### AC-008 - Support production incident review

- **Given** a production outage has been mitigated
- **When** the user creates a Production PIR with impact, timeline, people involved, detection method, resolved timestamp, 5 Whys, and actions
- **Then** the generated report contains all required PMIR sections and is suitable for stakeholder review.

### AC-009 - Search by problem statement and action

- **Given** multiple PIRs exist
- **When** the user runs `pirs search "retry state"`
- **Then** the system searches problem statements, timeline events, 5 Whys, root causes, and actions and prints matching PIRs with snippets.

### AC-010 - Validate repository health

- **Given** a PIR repository with a duplicate number, an overdue action, and a malformed timestamp
- **When** the user runs `pirs doctor`
- **Then** the system reports all three issues, exits with code `1` because of the duplicate or malformed timestamp, and classifies the overdue action as warning or error according to config.

### AC-011 - Export redacted JSON-PIR

- **Given** a PIR contains captured command output with a token-like string
- **When** the user runs `pirs export json --redact`
- **Then** the JSON output masks the token before writing to stdout.

### AC-012 - MCP incident logging

- **Given** an LLM client is connected to `pirs mcp serve`
- **When** it calls `log_incident` for a failing test with agent metadata and output summary
- **Then** the system creates or updates a PIR and records the agent as the actor.

---

## 8. Error Handling

| Error Condition | CLI Behaviour | Suggested Error |
|---|---|---|
| No PIR repository found | Exit non-zero unless command can use defaults | `PIR repository not found; run pirs init` |
| Missing problem statement | Exit non-zero in non-interactive mode | `problem statement is required` |
| Invalid PIR number | Exit non-zero | `invalid PIR number: <value>` |
| PIR not found | Exit non-zero | `PIR not found: <query>` |
| Ambiguous fuzzy lookup | Exit non-zero and list candidates | `ambiguous PIR query: <query>` |
| Invalid timestamp | Exit non-zero | `invalid timestamp for <field>` |
| Impossible duration | Exit non-zero or validation error | `resolved_at is before detected_at` |
| Missing action owner | Validation error | `action <id> requires an owner` |
| Review gate failed | Exit non-zero and list missing fields | `PIR is not ready for Reviewed` |
| Secret detected in output | Redact or fail depending on config | `captured output matched restricted pattern` |
| MCP HTTP unauthenticated on public bind | Warn or fail depending on config | `HTTP MCP requires authentication for non-loopback bind` |
| Concurrent write conflict | Retry or exit non-zero | `PIR repository changed during write; retry` |

---

## 9. Implementation TODO Checklist

### 9.1 Core Library

- [ ] Create `pirs-core` crate and public API surface.
- [ ] Define `Pir`, `IncidentStatus`, `IncidentSeverity`, `IncidentType`, `Actor`, `TimelineEvent`, `WhyEntry`, `ActionItem`, and `EvidenceLink` types.
- [ ] Implement slug generation and filename parsing.
- [ ] Implement config discovery for `pirs.toml`, `.pir-dir`, `$PIRS_CONFIG`, and `$PIR_DIRECTORY`.
- [ ] Implement Markdown + YAML frontmatter parser.
- [ ] Implement repository operations: `list`, `get`, `find`, `create`, `update_status`, `append_timeline`, `add_why`, `add_action`, `update_action`, `link_evidence`.
- [ ] Implement duration derivation and validation.
- [ ] Implement file locking or atomic create semantics.

### 9.2 CLI

- [ ] Scaffold `pirs` binary with `clap`.
- [ ] Add global flags `--cwd`, `--json`, `--quiet`, and `--no-color`.
- [ ] Implement commands: `init`, `new`, `run`, `timeline`, `why`, `action`, `actions`, `people`, `link`, `status`, `show`, `list`, `search`, `doctor`, `generate`, `metrics`, `export`, `import`, `template`, `config`, `mcp serve`.
- [ ] Ensure all write commands support non-interactive agent usage.
- [ ] Ensure diagnostics go to stderr and structured output goes to stdout.
- [ ] Add shell completions.

### 9.3 MCP

- [x] Implement MCP read tools: `list_pirs`, `get_pir`, `search_pirs`, `get_open_actions`, `get_repository_info`, `validate_pir`, `get_incident_metrics`, `suggest_related_pirs`.
- [ ] Implement MCP write tools: `create_pir`, `log_incident`, `append_timeline_event`, `update_status`, `add_why`, `add_action`, `update_action`, `link_evidence`, `finalize_review`.
- [ ] Record agent attribution for every MCP write.
- [ ] Add optional HTTP transport with bearer-token authentication.

### 9.4 Templates and Reports

- [ ] Add built-in templates for Development, Production, Security, Process, and Minimal PIRs.
- [ ] Add `pirs generate report <PIR>`.
- [ ] Add `pirs generate actions`.
- [ ] Add `pirs metrics`.
- [ ] Add blameless language warnings.

### 9.5 Export / Import / Schema

- [ ] Define `schema/json-pir/v1.json`.
- [ ] Implement single and bulk JSON-PIR export.
- [ ] Implement redacted export.
- [ ] Implement JSON-PIR import with dry-run and overwrite policies.

### 9.6 Testing

- [ ] Add CLI integration tests for all core commands.
- [ ] Add tests for agent-only PIR creation.
- [ ] Add tests for `pirs run --on-fail create` preserving command exit codes.
- [ ] Add tests for duration derivation and impossible duration validation.
- [ ] Add tests for review gate failures.
- [ ] Add tests for action registers and overdue action filtering.
- [ ] Add tests for secret redaction.
- [ ] Add MCP tool tests.
- [ ] Add fixture corpus for development, production, security, and process PIRs.

### 9.7 Documentation

- [ ] Write getting-started guide.
- [ ] Document the PIR file format.
- [ ] Document agent workflow patterns for LLMs.
- [ ] Document incident severity and status taxonomy.
- [ ] Document MCP security expectations.

---

## 10. Out of Scope for Initial Version

- Real-time incident paging or alerting.
- Replacing issue trackers such as Jira, Linear, or GitHub Issues.
- Full problem-management workflow beyond linking follow-up actions.
- Automatic root-cause analysis without human or agent confirmation.
- Hosted web application or multi-user server.
- Production log ingestion at large scale.

---

## 11. Open Questions

- [ ] Should the default directory be `doc/pir`, `doc/pirs`, or `.pirs/`?
`doc/pir` is more discoverable and git-friendly, while `.pirs/` is more hidden and protected from casual browsing. This spec proposes `doc/pir` with a `.gitignore` to exclude sensitive PIRs when needed.
- [ ] Should `time_to_resolve` mean `resolved_at - detected_at` or `resolved_at - occurred_at` in reports? This spec proposes `resolved_at - detected_at` and separately tracks total duration.
yes agreed, `time_to_resolve` should be `resolved_at - detected_at` to reflect the time spent in active response and remediation, while total duration can be derived from `resolved_at - occurred_at` for overall incident lifecycle analysis.
- [ ] Should every PIR require five 5 Whys entries, or should fewer entries be acceptable with an explicit waiver?
Optional 5 Whys entries with a warning if fewer than five are present strikes a good balance between encouraging thorough analysis and allowing flexibility for simpler incidents or when information is limited.
- [ ] Should agent-created development incidents default to `Low` or `Medium` severity?
`Low` severity for agent-created development incidents seems appropriate to avoid overwhelming the incident log with high-severity entries while still capturing valuable debugging information. `Medium` to `High` is determined by how close it is to production impact, and can be set by the agent or human based on context.
- [ ] Should `pirs run` create one PIR per failing command, or append to a session PIR by default?
One per failing command is simpler and provides clearer granularity for individual failures, while appending to a session PIR can group related failures together. This spec proposes creating a new PIR by default for clearer attribution and easier tracking, with an option to append to an existing PIR when desired.
- [ ] Should action owners be required to map to external identities, or can free-text owner names remain valid?
free-text owner names can remain valid for flexibility, especially for agent actors or when external identity mapping is not available. However, supporting optional external identity fields can enhance integration with team directories and accountability.
- [ ] Which fields should be considered sensitive by default for redacted export?
None by default to avoid over-redaction, but configurable patterns for common secrets like tokens, credentials, and private URLs should be supported and recommended.
- [ ] Should `pirs` integrate with git to attach commit SHA and dirty-worktree status automatically?
No integration, just a recommendation to include that information in the repository info tool and allow users to link commits as evidence when relevant.
- [ ] Should the MCP HTTP transport be disabled by default unless an auth token is configured?
No, it can be enabled by default for local use, but should warn when binding to a non-loopback address without authentication configured. Clear documentation on secure usage is essential.

---

## 12. Assumption Inventory

| # | Assumption | Confidence | Where it matters |
|---|---|---:|---|
| A-01 | `pirs` should inherit the library-first Rust architecture from `adrs`. | High | Workspace layout, CLI structure, MCP implementation. |
| A-02 | PIR records should be Markdown files with YAML frontmatter, not a database. | High | Version control, parser, export/import design. |
| A-03 | LLM agents are first-class users, not just indirect users through humans. | High | MCP tools, non-interactive CLI, agent actor fields. |
| A-04 | Small development failures are valid incidents when they teach something or consume debugging time. | High | `pirs run`, Development incident type, agent-only records. |
| A-05 | A fake bootstrap PIR would pollute the incident log, so `init` should avoid creating one. | Medium | Difference from `adrs init`. |
| A-06 | `time_to_discover` should be derived from occurrence to detection, while `time_to_resolve` should be derived from detection to resolution. | Medium | Metrics and reporting. |
| A-07 | File locking is needed because MCP and agent workflows can create concurrent writes. | High | Repository implementation. |
| A-08 | Blameless wording should be enforced through labels and optional language linting. | Medium | Templates, doctor checks. |
| A-09 | Captured command output may contain secrets and must be redacted before persistence. | High | `pirs run`, MCP logging, export. |
| A-10 | External integrations should begin as typed links rather than full API sync. | Medium | Scope control for initial version. |

---

## 13. Suggested Experiments

| Assumption to test | Experiment | Effort | Signal |
|---|---|---:|---|
| Agent-only incidents are useful | Ask an LLM coding agent to call `pirs run --on-fail create` during a failing test loop. | Low | Determines whether the workflow is low-friction. |
| Duration definitions are intuitive | Show sample reports to engineers and incident managers. | Low | Validates `time_to_resolve` and total duration wording. |
| 5 Whys gate is too strict | Try closing several real development PIRs with and without five entries. | Low | Reveals whether waivers are needed. |
| Redaction works | Run command-output fixtures containing tokens, URLs, and stack traces through redaction. | Medium | Confirms sensitive data does not persist. |
| File format is reviewable | Put several generated PIRs through git diff and code review. | Low | Confirms Markdown/YAML ergonomics. |
| MCP write safety | Run concurrent MCP `log_incident` calls. | Medium | Validates locking and duplicate handling. |

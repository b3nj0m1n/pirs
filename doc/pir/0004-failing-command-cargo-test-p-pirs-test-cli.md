---
number: 4
title: 'Failing command: cargo test -p pirs --test cli'
status: Open
severity: Low
incident_type: Development
problem_statement: |
  Wrapped command exited with code 101.

  Command: cargo test -p pirs --test cli

  exit_code: 101
  command: cargo test -p pirs --test cli
  --- stdout ---

  running 14 tests
  test ac_001_init_creates_pir_dir_without_sample_pir ... ok
  test ac_002_agent_only_development_incident ... ok
  test shows_help ... ok
  test ac_003_run_on_fail_creates_pir_and_propagates_exit_code ... ok
  test ac_007_review_gate_blocks_when_incomplete ... ok
  test ac_010_doctor_reports_clean_repo ... ok
  test ac_011_export_json_emits_schema_and_pir ... ok
  test ac_009_search_finds_problem_statement_text ... ok
  test ac_011_export_json_redact_masks_configured_patterns ... FAILED
  test import_json_stdin_dry_run_reports_without_writing ... FAILED
  test import_json_file_creates_pir_from_bulk_export ... FAILED
  test status_resolved_now_sets_resolved_at_and_duration ... ok
  test ac_005_006_why_and_action_add ... ok
  test import_json_skips_existing_number_unless_overwrite_is_supplied ... FAILED

  failures:

  ---- ac_011_export_json_redact_masks_configured_patterns stdout ----

  thread 'ac_011_export_json_redact_masks_configured_patterns' (790028) panicked at /private/tmp/rust-20251211-7744-a4uzq2/rustc-1.92.0-src/library/core/src/ops/function.rs:250:5:
  Unexpected failure.
  code=2
  stderr=``````
  error: unexpected argument \'--redact\' found

    tip: to pass \'--redact\' as a value, use \'-- --redact\'

  Usage: pirs export <FORMAT>

  For more information, try \'--help\'.
  ```
  ```
  command=`cd "/var/folders/x8/l20ldss173b761wbskztk0j40000gn/T/.tmpdgUALp" && "/Users/ben/IdeaProjects/pirs/target/debug/pirs" "export" "json" "--redact"`
  code=2
  stdout=""
  stderr=```
  error: unexpected argument \'--redact\' found

    tip: to pass \'--redact\' as a value, use \'-- --redact\'

  Usage: pirs export <FORMAT>

  For more information, try \'--help\'.
  ```


  note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

  ---- import_json_stdin_dry_run_reports_without_writing stdout ----

  thread 'import_json_stdin_dry_run_reports_without_writing' (790031) panicked at /private/tmp/rust-20251211-7744-a4uzq2/rustc-1.92.0-src/library/core/src/ops/function.rs:250:5:
  Unexpected failure.
  code=2
  stderr=``````
  error: unrecognized subcommand \'import\'

    tip: a similar subcommand exists: \'export\'

  Usage: pirs [OPTIONS] <COMMAND>

  For more information, try \'--help\'.
  ```
  ```
  command=`cd "/var/folders/x8/l20ldss173b761wbskztk0j40000gn/T/.tmpdAl7mx" && "/Users/ben/IdeaProjects/pirs/target/debug/pirs" "import" "json" "-" "--dry-run"`
  stdin=````
  {
    \"schema\": \"https://example.invalid/schema/json-pir/v1.json\",
    \"version\": \"1\",
    \"tool\": {
      \"name\": \"pirs\",
      \"version\": \"0.1.0\"
    },
    \"generated_at\": \"2026-04-26T05:55:04.217851Z\",
    \"repository\": {
      \"root\": \"/private/var/folders/x8/l20ldss173b761wbskztk0j40000gn/T/.tmpULvAzz\",
      \"pir_dir\": \"doc/pir\"
    },
    \"pirs\": [
      {
        \"number\": 1,
        \"title\": \"Dry run incident\",
        \"status\": \"Open\",
        \"severity\": \"Low\",
        \"incident_type\": \"Development\",
        \"problem_statement\": \"should not be written\",
        \"detected_at\": \"2026-04-26T05:55:04.197747Z\",
        \"timeline\": [
          {
            \"at\": \"2026-04-26T05:55:04.197747Z\",
            \"actor\": \"ben\",
            \"type\": \"detected\",
            \"description\": \"incident detected\"
          }
        ],
        \"impact\": \"_What systems, tests, environments, or workflows were affected?_\",
        \"confidentiality\": \"Internal\"
      }
    ]
  }
  ```
  `
  code=2
  stdout=""
  stderr=```
  error: unrecognized subcommand \'import\'

    tip: a similar subcommand exists: \'export\'

  Usage: pirs [OPTIONS] <COMMAND>

  For more information, try \'--help\'.
  ```



  ---- import_json_file_creates_pir_from_bulk_export stdout ----

  thread 'import_json_file_creates_pir_from_bulk_export' (790029) panicked at /private/tmp/rust-20251211-7744-a4uzq2/rustc-1.92.0-src/library/core/src/ops/function.rs:250:5:
  Unexpected failure.
  code=2
  stderr=``````
  error: unrecognized subcommand \'import\'

    tip: a similar subcommand exists: \'export\'

  Usage: pirs [OPTIONS] <COMMAND>

  For more information, try \'--help\'.
  ```
  ```
  command=`cd "/var/folders/x8/l20ldss173b761wbskztk0j40000gn/T/.tmpLV47Ri" && "/Users/ben/IdeaProjects/pirs/target/debug/pirs" "import" "json" "/var/folders/x8/l20ldss173b761wbskztk0j40000gn/T/.tmpLV47Ri/import.json"`
  code=2
  stdout=""
  stderr=```
  error: unrecognized subcommand \'import\'

    tip: a similar subcommand exists: \'export\'

  Usage: pirs [OPTIONS] <COMMAND>

  For more information, try \'--help\'.
  ```



  ---- import_json_skips_existing_number_unless_overwrite_is_supplied stdout ----

  thread 'import_json_skips_existing_number_unless_overwrite_is_supplied' (790030) panicked at /private/tmp/rust-20251211-7744-a4uzq2/rustc-1.92.0-src/library/core/src/ops/function.rs:250:5:
  Unexpected failure.
  code=2
  stderr=``````
  error: unrecognized subcommand \'import\'

    tip: a similar subcommand exists: \'export\'

  Usage: pirs [OPTIONS] <COMMAND>

  For more information, try \'--help\'.
  ```
  ```
  command=`cd "/var/folders/x8/l20ldss173b761wbskztk0j40000gn/T/.tmpm30IEt" && "/Users/ben/IdeaProjects/pirs/target/debug/pirs" "import" "json" "/var/folders/x8/l20ldss173b761wbskztk0j40000gn/T/.tmpm30IEt/import.json"`
  code=2
  stdout=""
  stderr=```
  error: unrecognized subcommand \'import\'

    tip: a similar subcommand exists: \'export\'

  Usage: pirs [OPTIONS] <COMMAND>

  For more information, try \'--help\'.
  ```




  failures:
      ac_011_export_json_redact_masks_configured_patterns
      import_json_file_creates_pir_from_bulk_export
      import_json_skips_existing_number_unless_overwrite_is_supplied
      import_json_stdin_dry_run_reports_without_writing

  test result: FAILED. 10 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.05s


  --- stderr ---
     Compiling pirs-core v0.1.0 (/Users/ben/IdeaProjects/pirs/crates/pirs-core)
     Compiling pirs v0.1.0 (/Users/ben/IdeaProjects/pirs/crates/pirs)
      Finished `test` profile [unoptimized + debuginfo] target(s) in 2.97s
       Running tests/cli.rs (target/debug/deps/cli-2fa527d27c06050e)
  error: test failed, to rerun pass `-p pirs --test cli`
occurred_at: 2026-04-26T05:54:59.505198Z
detected_at: 2026-04-26T05:55:04.268313Z
time_to_discover: PT4S
detection_method: agent-command-runner
people_involved:
- name: GitHub Copilot
  type: agent
timeline:
- at: 2026-04-26T05:55:04.268313Z
  actor: GitHub Copilot
  type: detected
  description: command failed (exit 101)
confidentiality: Internal
---

# 4. Failing command: cargo test -p pirs --test cli

> Type: Development · Severity: Low

## Problem Statement

Wrapped command exited with code 101.

Command: cargo test -p pirs --test cli

exit_code: 101
command: cargo test -p pirs --test cli
--- stdout ---

running 14 tests
test ac_001_init_creates_pir_dir_without_sample_pir ... ok
test ac_002_agent_only_development_incident ... ok
test shows_help ... ok
test ac_003_run_on_fail_creates_pir_and_propagates_exit_code ... ok
test ac_007_review_gate_blocks_when_incomplete ... ok
test ac_010_doctor_reports_clean_repo ... ok
test ac_011_export_json_emits_schema_and_pir ... ok
test ac_009_search_finds_problem_statement_text ... ok
test ac_011_export_json_redact_masks_configured_patterns ... FAILED
test import_json_stdin_dry_run_reports_without_writing ... FAILED
test import_json_file_creates_pir_from_bulk_export ... FAILED
test status_resolved_now_sets_resolved_at_and_duration ... ok
test ac_005_006_why_and_action_add ... ok
test import_json_skips_existing_number_unless_overwrite_is_supplied ... FAILED

failures:

---- ac_011_export_json_redact_masks_configured_patterns stdout ----

thread 'ac_011_export_json_redact_masks_configured_patterns' (790028) panicked at /private/tmp/rust-20251211-7744-a4uzq2/rustc-1.92.0-src/library/core/src/ops/function.rs:250:5:
Unexpected failure.
code=2
stderr=``````
error: unexpected argument \'--redact\' found

  tip: to pass \'--redact\' as a value, use \'-- --redact\'

Usage: pirs export <FORMAT>

For more information, try \'--help\'.
```
```
command=`cd "/var/folders/x8/l20ldss173b761wbskztk0j40000gn/T/.tmpdgUALp" && "/Users/ben/IdeaProjects/pirs/target/debug/pirs" "export" "json" "--redact"`
code=2
stdout=""
stderr=```
error: unexpected argument \'--redact\' found

  tip: to pass \'--redact\' as a value, use \'-- --redact\'

Usage: pirs export <FORMAT>

For more information, try \'--help\'.
```


note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- import_json_stdin_dry_run_reports_without_writing stdout ----

thread 'import_json_stdin_dry_run_reports_without_writing' (790031) panicked at /private/tmp/rust-20251211-7744-a4uzq2/rustc-1.92.0-src/library/core/src/ops/function.rs:250:5:
Unexpected failure.
code=2
stderr=``````
error: unrecognized subcommand \'import\'

  tip: a similar subcommand exists: \'export\'

Usage: pirs [OPTIONS] <COMMAND>

For more information, try \'--help\'.
```
```
command=`cd "/var/folders/x8/l20ldss173b761wbskztk0j40000gn/T/.tmpdAl7mx" && "/Users/ben/IdeaProjects/pirs/target/debug/pirs" "import" "json" "-" "--dry-run"`
stdin=````
{
  \"schema\": \"https://example.invalid/schema/json-pir/v1.json\",
  \"version\": \"1\",
  \"tool\": {
    \"name\": \"pirs\",
    \"version\": \"0.1.0\"
  },
  \"generated_at\": \"2026-04-26T05:55:04.217851Z\",
  \"repository\": {
    \"root\": \"/private/var/folders/x8/l20ldss173b761wbskztk0j40000gn/T/.tmpULvAzz\",
    \"pir_dir\": \"doc/pir\"
  },
  \"pirs\": [
    {
      \"number\": 1,
      \"title\": \"Dry run incident\",
      \"status\": \"Open\",
      \"severity\": \"Low\",
      \"incident_type\": \"Development\",
      \"problem_statement\": \"should not be written\",
      \"detected_at\": \"2026-04-26T05:55:04.197747Z\",
      \"timeline\": [
        {
          \"at\": \"2026-04-26T05:55:04.197747Z\",
          \"actor\": \"ben\",
          \"type\": \"detected\",
          \"description\": \"incident detected\"
        }
      ],
      \"impact\": \"_What systems, tests, environments, or workflows were affected?_\",
      \"confidentiality\": \"Internal\"
    }
  ]
}
```
`
code=2
stdout=""
stderr=```
error: unrecognized subcommand \'import\'

  tip: a similar subcommand exists: \'export\'

Usage: pirs [OPTIONS] <COMMAND>

For more information, try \'--help\'.
```



---- import_json_file_creates_pir_from_bulk_export stdout ----

thread 'import_json_file_creates_pir_from_bulk_export' (790029) panicked at /private/tmp/rust-20251211-7744-a4uzq2/rustc-1.92.0-src/library/core/src/ops/function.rs:250:5:
Unexpected failure.
code=2
stderr=``````
error: unrecognized subcommand \'import\'

  tip: a similar subcommand exists: \'export\'

Usage: pirs [OPTIONS] <COMMAND>

For more information, try \'--help\'.
```
```
command=`cd "/var/folders/x8/l20ldss173b761wbskztk0j40000gn/T/.tmpLV47Ri" && "/Users/ben/IdeaProjects/pirs/target/debug/pirs" "import" "json" "/var/folders/x8/l20ldss173b761wbskztk0j40000gn/T/.tmpLV47Ri/import.json"`
code=2
stdout=""
stderr=```
error: unrecognized subcommand \'import\'

  tip: a similar subcommand exists: \'export\'

Usage: pirs [OPTIONS] <COMMAND>

For more information, try \'--help\'.
```



---- import_json_skips_existing_number_unless_overwrite_is_supplied stdout ----

thread 'import_json_skips_existing_number_unless_overwrite_is_supplied' (790030) panicked at /private/tmp/rust-20251211-7744-a4uzq2/rustc-1.92.0-src/library/core/src/ops/function.rs:250:5:
Unexpected failure.
code=2
stderr=``````
error: unrecognized subcommand \'import\'

  tip: a similar subcommand exists: \'export\'

Usage: pirs [OPTIONS] <COMMAND>

For more information, try \'--help\'.
```
```
command=`cd "/var/folders/x8/l20ldss173b761wbskztk0j40000gn/T/.tmpm30IEt" && "/Users/ben/IdeaProjects/pirs/target/debug/pirs" "import" "json" "/var/folders/x8/l20ldss173b761wbskztk0j40000gn/T/.tmpm30IEt/import.json"`
code=2
stdout=""
stderr=```
error: unrecognized subcommand \'import\'

  tip: a similar subcommand exists: \'export\'

Usage: pirs [OPTIONS] <COMMAND>

For more information, try \'--help\'.
```




failures:
    ac_011_export_json_redact_masks_configured_patterns
    import_json_file_creates_pir_from_bulk_export
    import_json_skips_existing_number_unless_overwrite_is_supplied
    import_json_stdin_dry_run_reports_without_writing

test result: FAILED. 10 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.05s


--- stderr ---
   Compiling pirs-core v0.1.0 (/Users/ben/IdeaProjects/pirs/crates/pirs-core)
   Compiling pirs v0.1.0 (/Users/ben/IdeaProjects/pirs/crates/pirs)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.97s
     Running tests/cli.rs (target/debug/deps/cli-2fa527d27c06050e)
error: test failed, to rerun pass `-p pirs --test cli`


## Impact

_What systems, tests, environments, or workflows were affected?_

## People and Systems Involved

_Humans, agents, teams, or systems involved (blameless)._

## Timeline

_Ordered events, populated via `pirs timeline add`._

## Detection and Resolution Timing

_Time to discover and time to resolve, derived from timestamps._

## 5 Whys

_Add ordered entries via `pirs why add`._

## Actions

_Add follow-up actions via `pirs action add`._

## Lessons Learned

_What went well, what went wrong, where we got lucky._

## Links

_Typed evidence links: commits, PRs, issues, dashboards, runbooks._

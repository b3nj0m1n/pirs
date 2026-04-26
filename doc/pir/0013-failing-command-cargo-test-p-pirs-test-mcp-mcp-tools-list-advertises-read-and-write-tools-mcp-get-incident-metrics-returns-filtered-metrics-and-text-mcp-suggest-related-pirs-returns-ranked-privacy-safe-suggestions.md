---
number: 13
title: 'Failing command: cargo test -p pirs --test mcp mcp_tools_list_advertises_read_and_write_tools mcp_get_incident_metrics_returns_filtered_metrics_and_text mcp_suggest_related_pirs_returns_ranked_privacy_safe_suggestions'
status: Resolved
severity: Low
incident_type: Development
problem_statement: "Wrapped command exited with code 1.\n\nCommand: cargo test -p pirs --test mcp mcp_tools_list_advertises_read_and_write_tools mcp_get_incident_metrics_returns_filtered_metrics_and_text mcp_suggest_related_pirs_returns_ranked_privacy_safe_suggestions\n\nexit_code: 1\ncommand: cargo test -p pirs --test mcp mcp_tools_list_advertises_read_and_write_tools mcp_get_incident_metrics_returns_filtered_metrics_and_text mcp_suggest_related_pirs_returns_ranked_privacy_safe_suggestions\n--- stdout "
resolved_at: 2026-04-26T08:16:51.172854Z
timeline:
- at: 2026-04-26T08:16:51.137416Z
  actor: GitHub Copilot
  type: investigated
  description: The cargo test command supplied three test-name filters, but cargo accepts only one. Recovered by using the shared mcp_ filter, which ran all MCP integration tests successfully.
- at: 2026-04-26T08:16:51.172854Z
  actor: pirs
  type: resolved
  description: status -> Resolved
five_whys:
- question: Why did the MCP verification command fail?
  answer: The command used multiple cargo test-name filters instead of a single filter string.
impact: _What systems, tests, environments, or workflows were affected?_
root_cause: The command used multiple cargo test-name filters instead of a single filter string.
confidentiality: Internal
---

  --- stderr ---
  error: unexpected argument 'mcp_get_incident_metrics_returns_filtered_metrics_and_text' found

  Usage: cargo test [OPTIONS] [TESTNAME] [-- [ARGS]...]

  For more information, try '--help'.
occurred_at: 2026-04-26T08:16:26.930106Z
detected_at: 2026-04-26T08:16:26.959396Z
time_to_discover: PT0S
detection_method: agent-command-runner
people_involved:
- name: GitHub Copilot
  type: agent
timeline:
- at: 2026-04-26T08:16:26.959396Z
  actor: GitHub Copilot
  type: detected
  description: command failed (exit 1)
confidentiality: Internal
---

# 13. Failing command: cargo test -p pirs --test mcp mcp_tools_list_advertises_read_and_write_tools mcp_get_incident_metrics_returns_filtered_metrics_and_text mcp_suggest_related_pirs_returns_ranked_privacy_safe_suggestions

> Type: Development · Severity: Low

## Problem Statement

Wrapped command exited with code 1.

Command: cargo test -p pirs --test mcp mcp_tools_list_advertises_read_and_write_tools mcp_get_incident_metrics_returns_filtered_metrics_and_text mcp_suggest_related_pirs_returns_ranked_privacy_safe_suggestions

exit_code: 1
command: cargo test -p pirs --test mcp mcp_tools_list_advertises_read_and_write_tools mcp_get_incident_metrics_returns_filtered_metrics_and_text mcp_suggest_related_pirs_returns_ranked_privacy_safe_suggestions
--- stdout ---

--- stderr ---
error: unexpected argument 'mcp_get_incident_metrics_returns_filtered_metrics_and_text' found

Usage: cargo test [OPTIONS] [TESTNAME] [-- [ARGS]...]

For more information, try '--help'.


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





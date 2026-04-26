//! MCP (Model Context Protocol) server for the PIR repository.
//!
//! Exposes the [`Repository`] API as MCP tools so LLM agents can manage PIRs
//! directly through their MCP client without parsing CLI output.
//!
//! Implements REQ-MCP-001 through REQ-MCP-006 from `spec/pirs_requirements_spec.md`:
//!
//! * Stdio transport by default; HTTP transport gated on the `http` cargo
//!   feature (REQ-MCP-001).
//! * Each tool call opens a fresh [`Repository`] (REQ-MCP-002).
//! * Read tools: `list_pirs`, `get_pir`, `search_pirs`, `get_open_actions`,
//!   `get_repository_info`, `validate_pir` (REQ-MCP-003).
//! * Write tools: `create_pir`, `append_timeline_event`, `update_status`,
//!   `add_why`, `add_action`, `update_action`, `link_evidence` (REQ-MCP-004).
//! * Agent attribution via the server-level `--agent` flag (REQ-MCP-005).

use anyhow::{Context, Result, anyhow};
use pirs_core::{
    ActionItem, ActionStatus, Actor, ActorKind, EvidenceLink, IncidentSeverity, IncidentStatus,
    IncidentType, LinkKind, Pir, Repository, TimelineEvent, TimelineEventType, WhyEntry, lint,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use tower_mcp::extract::{Json, State};
use tower_mcp::{CallToolResult, McpRouter, StdioTransport, ToolBuilder};

const SERVER_NAME: &str = "pirs";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Shared state passed to every tool handler.
#[derive(Debug, Clone)]
pub struct PirState {
    pub root: PathBuf,
    /// Default agent identifier used when a write tool does not specify one.
    pub agent: Option<String>,
}

/// Run the MCP server. Blocks until stdin is closed (stdio) or the HTTP server
/// shuts down.
///
/// `http_addr` is only honoured when the crate is built with the `http`
/// feature; otherwise this function returns an error if a value is supplied.
pub fn serve(state: PirState, http_addr: Option<String>) -> Result<()> {
    // Diagnostics go to stderr so they never collide with the stdio transport.
    let _ = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("pirs=info,tower_mcp=warn")),
        )
        .try_init();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("failed to build tokio runtime")?;

    runtime.block_on(async move { run_async(state, http_addr).await })
}

async fn run_async(state: PirState, http_addr: Option<String>) -> Result<()> {
    let router = build_router(state);

    if let Some(addr) = http_addr {
        return run_http(router, addr).await;
    }

    StdioTransport::new(router)
        .run()
        .await
        .map_err(|e| anyhow!("MCP stdio transport error: {e}"))
}

#[cfg(feature = "http")]
async fn run_http(router: McpRouter, addr: String) -> Result<()> {
    use tower_mcp::transport::HttpTransport;

    if !is_loopback(&addr) {
        tracing::warn!(
            address = %addr,
            "MCP HTTP transport bound to non-loopback address without authentication; \
             see REQ-MCP-006"
        );
    }
    HttpTransport::new(router)
        .serve(&addr)
        .await
        .map_err(|e| anyhow!("MCP HTTP transport error: {e}"))
}

#[cfg(not(feature = "http"))]
async fn run_http(_router: McpRouter, _addr: String) -> Result<()> {
    Err(anyhow!(
        "this build of pirs was compiled without the `http` feature; \
         rebuild with `--features http` to enable HTTP MCP transport"
    ))
}

#[cfg(feature = "http")]
fn is_loopback(addr: &str) -> bool {
    addr.starts_with("127.")
        || addr.starts_with("[::1]")
        || addr.starts_with("localhost")
        || addr.starts_with("[::1")
}

// ---------------------------------------------------------------------------
// Router construction
// ---------------------------------------------------------------------------

fn build_router(state: PirState) -> McpRouter {
    let st = Arc::new(state);

    McpRouter::new()
        .server_info(SERVER_NAME, SERVER_VERSION)
        // --- read tools (REQ-MCP-003) ---
        .tool(tool_list_pirs(st.clone()))
        .tool(tool_get_pir(st.clone()))
        .tool(tool_search_pirs(st.clone()))
        .tool(tool_get_open_actions(st.clone()))
        .tool(tool_get_repository_info(st.clone()))
        .tool(tool_validate_pir(st.clone()))
        // --- write tools (REQ-MCP-004) ---
        .tool(tool_create_pir(st.clone()))
        .tool(tool_append_timeline_event(st.clone()))
        .tool(tool_update_status(st.clone()))
        .tool(tool_add_why(st.clone()))
        .tool(tool_add_action(st.clone()))
        .tool(tool_update_action(st.clone()))
        .tool(tool_link_evidence(st))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// REQ-MCP-002: open the repository fresh for every tool invocation.
fn open_repo(state: &PirState) -> Result<Repository> {
    Repository::open(&state.root)
        .with_context(|| format!("PIR repository not found at {}", state.root.display()))
}

fn now_or_parse(at: Option<&str>) -> Result<OffsetDateTime> {
    match at {
        Some(s) => OffsetDateTime::parse(s, &Rfc3339)
            .with_context(|| format!("invalid RFC3339 timestamp: {s}")),
        None => Ok(OffsetDateTime::now_utc()),
    }
}

fn pir_summary(p: &Pir) -> Value {
    json!({
        "number": p.number,
        "title": p.title,
        "status": p.status.to_string(),
        "severity": p.severity.to_string(),
        "incident_type": p.incident_type.to_string(),
        "tags": p.tags,
        "open_actions": p.actions.iter().filter(|a| !matches!(a.status, ActionStatus::Done | ActionStatus::Cancelled)).count(),
    })
}

fn ok_json(value: Value) -> tower_mcp::Result<CallToolResult> {
    let text = serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string());
    Ok(CallToolResult::text(text))
}

fn err_result<E: std::fmt::Display>(e: E) -> tower_mcp::Result<CallToolResult> {
    Ok(CallToolResult::error(e.to_string()))
}

// ---------------------------------------------------------------------------
// Read tools
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema, Default)]
struct ListPirsInput {
    /// Filter by status (open, investigating, mitigated, resolved, reviewed, cancelled).
    #[serde(default)]
    status: Option<String>,
    /// Filter by severity (low, medium, high, critical).
    #[serde(default)]
    severity: Option<String>,
    /// Filter by incident type (development, production, security, process).
    #[serde(default)]
    incident_type: Option<String>,
    /// Filter by tag.
    #[serde(default)]
    tag: Option<String>,
    /// Only PIRs with at least one open action.
    #[serde(default)]
    has_open_actions: bool,
}

fn tool_list_pirs(state: Arc<PirState>) -> tower_mcp::Tool {
    ToolBuilder::new("list_pirs")
        .title("List PIRs")
        .description("List all Post-Incident Reviews with optional filters.")
        .extractor_handler(
            state,
            |State(st): State<Arc<PirState>>, Json(input): Json<ListPirsInput>| async move {
                let repo = match open_repo(&st) {
                    Ok(r) => r,
                    Err(e) => return err_result(e),
                };
                let pirs = match repo.list() {
                    Ok(p) => p,
                    Err(e) => return err_result(e),
                };
                let want_status = input
                    .status
                    .as_deref()
                    .map(|s| IncidentStatus::from_str(s).unwrap());
                let want_sev = input
                    .severity
                    .as_deref()
                    .map(|s| IncidentSeverity::from_str(s).unwrap());
                let want_type = input
                    .incident_type
                    .as_deref()
                    .map(|s| IncidentType::from_str(s).unwrap());

                let filtered: Vec<Value> = pirs
                    .iter()
                    .filter(|p| want_status.as_ref().is_none_or(|s| &p.status == s))
                    .filter(|p| want_sev.as_ref().is_none_or(|s| &p.severity == s))
                    .filter(|p| want_type.as_ref().is_none_or(|t| &p.incident_type == t))
                    .filter(|p| {
                        input
                            .tag
                            .as_ref()
                            .is_none_or(|t| p.tags.iter().any(|x| x == t))
                    })
                    .filter(|p| {
                        !input.has_open_actions
                            || p.actions.iter().any(|a| {
                                !matches!(a.status, ActionStatus::Done | ActionStatus::Cancelled)
                            })
                    })
                    .map(pir_summary)
                    .collect();
                ok_json(json!({ "count": filtered.len(), "pirs": filtered }))
            },
        )
        .build()
}

#[derive(Debug, Deserialize, JsonSchema)]
struct GetPirInput {
    /// PIR number or fuzzy title query.
    query: String,
}

fn tool_get_pir(state: Arc<PirState>) -> tower_mcp::Tool {
    ToolBuilder::new("get_pir")
        .title("Get PIR")
        .description("Fetch a single PIR by number or fuzzy title query.")
        .extractor_handler(
            state,
            |State(st): State<Arc<PirState>>, Json(input): Json<GetPirInput>| async move {
                let repo = match open_repo(&st) {
                    Ok(r) => r,
                    Err(e) => return err_result(e),
                };
                match repo.find(&input.query) {
                    Ok(p) => match serde_json::to_value(&p) {
                        Ok(value) => ok_json(value),
                        Err(e) => err_result(anyhow!("failed to serialize PIR result: {e}")),
                    },
                    Err(e) => err_result(e),
                }
            },
        )
        .build()
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SearchPirsInput {
    /// Substring or fuzzy term to search for in titles and problem statements.
    query: String,
    #[serde(default)]
    case_sensitive: bool,
}

fn tool_search_pirs(state: Arc<PirState>) -> tower_mcp::Tool {
    ToolBuilder::new("search_pirs")
        .title("Search PIRs")
        .description("Search PIRs by substring match across title and problem statement.")
        .extractor_handler(
            state,
            |State(st): State<Arc<PirState>>, Json(input): Json<SearchPirsInput>| async move {
                let repo = match open_repo(&st) {
                    Ok(r) => r,
                    Err(e) => return err_result(e),
                };
                let pirs = match repo.list() {
                    Ok(p) => p,
                    Err(e) => return err_result(e),
                };
                let needle = if input.case_sensitive {
                    input.query.clone()
                } else {
                    input.query.to_lowercase()
                };
                let hits: Vec<Value> = pirs
                    .iter()
                    .filter(|p| {
                        let hay = format!("{} {}", p.title, p.problem_statement);
                        let hay = if input.case_sensitive {
                            hay
                        } else {
                            hay.to_lowercase()
                        };
                        hay.contains(&needle)
                    })
                    .map(pir_summary)
                    .collect();
                ok_json(json!({ "count": hits.len(), "pirs": hits }))
            },
        )
        .build()
}

#[derive(Debug, Deserialize, JsonSchema, Default)]
struct OpenActionsInput {
    /// Restrict to actions owned by this name.
    #[serde(default)]
    owner: Option<String>,
    /// Only include overdue actions.
    #[serde(default)]
    overdue: bool,
}

fn tool_get_open_actions(state: Arc<PirState>) -> tower_mcp::Tool {
    ToolBuilder::new("get_open_actions")
        .title("Get open actions")
        .description("List action items that are still open across the repository.")
        .extractor_handler(
            state,
            |State(st): State<Arc<PirState>>, Json(input): Json<OpenActionsInput>| async move {
                let repo = match open_repo(&st) {
                    Ok(r) => r,
                    Err(e) => return err_result(e),
                };
                let pirs = match repo.list() {
                    Ok(p) => p,
                    Err(e) => return err_result(e),
                };
                let today = OffsetDateTime::now_utc().date();
                let mut out: Vec<Value> = Vec::new();
                for p in &pirs {
                    for a in &p.actions {
                        if matches!(a.status, ActionStatus::Done | ActionStatus::Cancelled) {
                            continue;
                        }
                        if let Some(o) = &input.owner
                            && &a.owner != o
                        {
                            continue;
                        }
                        if input.overdue {
                            let Some(due) = &a.due else { continue };
                            let Ok(d) = time::Date::parse(
                                due,
                                &time::format_description::well_known::Iso8601::DATE,
                            ) else {
                                continue;
                            };
                            if d >= today {
                                continue;
                            }
                        }
                        out.push(json!({
                            "pir": p.number,
                            "id": a.id,
                            "description": a.description,
                            "owner": a.owner,
                            "owner_type": a.owner_type.to_string(),
                            "status": a.status.to_string(),
                            "due": a.due,
                        }));
                    }
                }
                ok_json(json!({ "count": out.len(), "actions": out }))
            },
        )
        .build()
}

#[derive(Debug, Deserialize, JsonSchema, Default)]
struct EmptyInput {}

fn tool_get_repository_info(state: Arc<PirState>) -> tower_mcp::Tool {
    ToolBuilder::new("get_repository_info")
        .title("Get repository info")
        .description("Return the resolved root, PIR directory, total count, and configuration.")
        .extractor_handler(state, |State(st): State<Arc<PirState>>, Json(_): Json<EmptyInput>| async move {
            let repo = match open_repo(&st) {
                Ok(r) => r,
                Err(e) => return err_result(e),
            };
            let count = match repo.list() {
                Ok(pirs) => pirs.len(),
                Err(e) => return err_result(e),
            };
            ok_json(json!({
                "root": repo.root().display().to_string(),
                "pir_dir": repo.config().pir_dir.display().to_string(),
                "pir_path": repo.pir_path().display().to_string(),
                "total_pirs": count,
                "templates": {
                    "default": repo.config().templates.default,
                    "custom": repo.config().templates.custom.as_ref().map(|p| p.display().to_string()),
                },
                "agent_default": st.agent,
            }))
        })
        .build()
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ValidatePirInput {
    /// PIR number to validate.
    pir: u32,
}

fn tool_validate_pir(state: Arc<PirState>) -> tower_mcp::Tool {
    ToolBuilder::new("validate_pir")
        .title("Validate PIR")
        .description("Run lint and review-gate checks against a PIR.")
        .extractor_handler(
            state,
            |State(st): State<Arc<PirState>>, Json(input): Json<ValidatePirInput>| async move {
                let repo = match open_repo(&st) {
                    Ok(r) => r,
                    Err(e) => return err_result(e),
                };
                let pir = match repo.get(input.pir) {
                    Ok(p) => p,
                    Err(e) => return err_result(e),
                };
                let issues: Vec<Value> = lint::lint_pir(&pir)
                    .iter()
                    .map(|i| {
                        let sev = match i.severity {
                            lint::IssueSeverity::Error => "error",
                            lint::IssueSeverity::Warning => "warning",
                            lint::IssueSeverity::Info => "info",
                        };
                        json!({ "severity": sev, "message": i.message })
                    })
                    .collect();
                let review_missing = lint::review_gate(&pir);
                ok_json(json!({
                    "pir": pir.number,
                    "issues": issues,
                    "review_gate_missing": review_missing,
                    "ready_for_review": review_missing.is_empty(),
                }))
            },
        )
        .build()
}

// ---------------------------------------------------------------------------
// Write tools
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct CreatePirInput {
    title: String,
    /// Required problem statement (REQ-NEW-002).
    problem_statement: String,
    /// development | production | security | process.
    #[serde(default)]
    incident_type: Option<String>,
    /// low | medium | high | critical.
    #[serde(default)]
    severity: Option<String>,
    /// Agent name; falls back to the server-level default agent.
    #[serde(default)]
    agent: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    /// Suppress the initial `detected` timeline event.
    #[serde(default)]
    no_initial_event: bool,
}

fn tool_create_pir(state: Arc<PirState>) -> tower_mcp::Tool {
    ToolBuilder::new("create_pir")
        .title("Create PIR")
        .description("Create a new Post-Incident Review.")
        .extractor_handler(
            state,
            |State(st): State<Arc<PirState>>, Json(input): Json<CreatePirInput>| async move {
                let repo = match open_repo(&st) {
                    Ok(r) => r,
                    Err(e) => return err_result(e),
                };
                let number = match repo.next_number() {
                    Ok(n) => n,
                    Err(e) => return err_result(e),
                };
                let incident_type = input
                    .incident_type
                    .as_deref()
                    .map(|s| IncidentType::from_str(s).unwrap())
                    .unwrap_or(IncidentType::Development);
                let severity = input
                    .severity
                    .as_deref()
                    .map(|s| IncidentSeverity::from_str(s).unwrap())
                    .unwrap_or(IncidentSeverity::Low);

                let agent = input.agent.clone().or_else(|| st.agent.clone());
                let mut pir = Pir::new(number, &input.title);
                pir.problem_statement = input.problem_statement;
                pir.incident_type = incident_type;
                pir.severity = severity;
                pir.tags = input.tags;
                pir.status = IncidentStatus::Open;
                let now = OffsetDateTime::now_utc();
                pir.detected_at = Some(now);
                if let Some(name) = agent.clone() {
                    pir.people_involved.push(Actor::agent(name));
                }
                if !input.no_initial_event {
                    let actor = pir
                        .people_involved
                        .first()
                        .map(|a| a.name.clone())
                        .unwrap_or_else(|| "mcp".into());
                    pir.timeline.push(TimelineEvent {
                        at: now,
                        actor,
                        event_type: TimelineEventType::Detected,
                        description: Some("incident detected".into()),
                    });
                }
                pir.recompute_durations();
                let path = match repo.create(&pir) {
                    Ok(p) => p,
                    Err(e) => return err_result(e),
                };
                ok_json(json!({
                    "number": pir.number,
                    "path": path.display().to_string(),
                    "title": pir.title,
                }))
            },
        )
        .build()
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AppendTimelineInput {
    pir: u32,
    /// RFC3339 timestamp; defaults to now.
    #[serde(default)]
    at: Option<String>,
    /// Actor name; falls back to the server-level default agent.
    #[serde(default)]
    actor: Option<String>,
    /// detected | investigated | mitigated | resolved | communicated | escalated | note.
    #[serde(default = "default_event_type")]
    event_type: String,
    message: String,
}

fn default_event_type() -> String {
    "note".into()
}

fn tool_append_timeline_event(state: Arc<PirState>) -> tower_mcp::Tool {
    ToolBuilder::new("append_timeline_event")
        .title("Append timeline event")
        .description("Append a typed timeline event to an existing PIR.")
        .extractor_handler(
            state,
            |State(st): State<Arc<PirState>>, Json(input): Json<AppendTimelineInput>| async move {
                let repo = match open_repo(&st) {
                    Ok(r) => r,
                    Err(e) => return err_result(e),
                };
                let at = match now_or_parse(input.at.as_deref()) {
                    Ok(t) => t,
                    Err(e) => return err_result(e),
                };
                let actor = input
                    .actor
                    .clone()
                    .or_else(|| st.agent.clone())
                    .unwrap_or_else(|| "mcp".into());
                let event = TimelineEvent {
                    at,
                    actor,
                    event_type: TimelineEventType::from_str(&input.event_type).unwrap(),
                    description: Some(input.message),
                };
                match repo.append_timeline(input.pir, event) {
                    Ok(()) => ok_json(json!({ "pir": input.pir, "ok": true })),
                    Err(e) => err_result(e),
                }
            },
        )
        .build()
}

#[derive(Debug, Deserialize, JsonSchema)]
struct UpdateStatusInput {
    pir: u32,
    /// open | investigating | mitigated | resolved | reviewed | cancelled.
    status: String,
    /// Set `resolved_at` to now when transitioning to Resolved.
    #[serde(default)]
    now: bool,
    #[serde(default)]
    cancellation_reason: Option<String>,
}

fn tool_update_status(state: Arc<PirState>) -> tower_mcp::Tool {
    ToolBuilder::new("update_status")
        .title("Update status")
        .description("Transition a PIR's status with the same gates the CLI enforces.")
        .extractor_handler(
            state,
            |State(st): State<Arc<PirState>>, Json(input): Json<UpdateStatusInput>| async move {
                let repo = match open_repo(&st) {
                    Ok(r) => r,
                    Err(e) => return err_result(e),
                };
                let status = IncidentStatus::from_str(&input.status).unwrap();
                let now = if input.now {
                    Some(OffsetDateTime::now_utc())
                } else {
                    None
                };
                match repo.update_status(input.pir, status.clone(), now, input.cancellation_reason)
                {
                    Ok(()) => ok_json(json!({
                        "pir": input.pir,
                        "status": status.to_string(),
                        "ok": true,
                    })),
                    Err(e) => err_result(e),
                }
            },
        )
        .build()
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AddWhyInput {
    pir: u32,
    question: String,
    answer: String,
    /// Promote the answer to root cause.
    #[serde(default)]
    as_root_cause: bool,
}

fn tool_add_why(state: Arc<PirState>) -> tower_mcp::Tool {
    ToolBuilder::new("add_why")
        .title("Add 5 Whys entry")
        .description("Append a 5 Whys entry to a PIR; optionally tag the answer as root cause.")
        .extractor_handler(
            state,
            |State(st): State<Arc<PirState>>, Json(input): Json<AddWhyInput>| async move {
                let repo = match open_repo(&st) {
                    Ok(r) => r,
                    Err(e) => return err_result(e),
                };
                if let Err(e) = repo.add_why(
                    input.pir,
                    WhyEntry {
                        question: input.question,
                        answer: input.answer.clone(),
                    },
                ) {
                    return err_result(e);
                }
                if input.as_root_cause {
                    let mut pir = match repo.get(input.pir) {
                        Ok(p) => p,
                        Err(e) => return err_result(e),
                    };
                    pir.root_cause = Some(input.answer);
                    if let Err(e) = repo.save(&mut pir) {
                        return err_result(e);
                    }
                }
                ok_json(json!({ "pir": input.pir, "ok": true }))
            },
        )
        .build()
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AddActionInput {
    pir: u32,
    description: String,
    owner: String,
    /// human | agent | team | system.
    #[serde(default)]
    owner_type: Option<String>,
    /// ISO-8601 due date (YYYY-MM-DD).
    #[serde(default)]
    due: Option<String>,
}

fn tool_add_action(state: Arc<PirState>) -> tower_mcp::Tool {
    ToolBuilder::new("add_action")
        .title("Add action item")
        .description("Add a follow-up action item to a PIR. Returns the new action id.")
        .extractor_handler(
            state,
            |State(st): State<Arc<PirState>>, Json(input): Json<AddActionInput>| async move {
                let repo = match open_repo(&st) {
                    Ok(r) => r,
                    Err(e) => return err_result(e),
                };
                let owner_type = input
                    .owner_type
                    .as_deref()
                    .map(|s| ActorKind::from_str(s).unwrap())
                    .unwrap_or(ActorKind::Human);
                let action = ActionItem {
                    id: String::new(),
                    description: input.description,
                    owner: input.owner,
                    owner_type,
                    due: input.due,
                    status: ActionStatus::Open,
                    evidence: Vec::new(),
                    notes: None,
                };
                match repo.add_action(input.pir, action) {
                    Ok(id) => ok_json(json!({ "pir": input.pir, "action_id": id })),
                    Err(e) => err_result(e),
                }
            },
        )
        .build()
}

#[derive(Debug, Deserialize, JsonSchema)]
struct UpdateActionInput {
    pir: u32,
    action_id: String,
    /// open | inprogress | blocked | done | cancelled.
    status: String,
    #[serde(default)]
    evidence: Vec<String>,
}

fn tool_update_action(state: Arc<PirState>) -> tower_mcp::Tool {
    ToolBuilder::new("update_action")
        .title("Update action item")
        .description("Update an action item's status and append evidence.")
        .extractor_handler(
            state,
            |State(st): State<Arc<PirState>>, Json(input): Json<UpdateActionInput>| async move {
                let repo = match open_repo(&st) {
                    Ok(r) => r,
                    Err(e) => return err_result(e),
                };
                let status = ActionStatus::from_str(&input.status).unwrap();
                match repo.update_action_status(input.pir, &input.action_id, status, input.evidence)
                {
                    Ok(()) => ok_json(
                        json!({ "pir": input.pir, "action_id": input.action_id, "ok": true }),
                    ),
                    Err(e) => err_result(e),
                }
            },
        )
        .build()
}

#[derive(Debug, Deserialize, JsonSchema)]
struct LinkEvidenceInput {
    pir: u32,
    uri: String,
    /// Commit | PullRequest | Issue | Log | Dashboard | Runbook | Document | RelatedTo | Other.
    #[serde(default = "default_link_kind")]
    kind: String,
    #[serde(default)]
    description: Option<String>,
}

fn default_link_kind() -> String {
    "RelatedTo".into()
}

fn tool_link_evidence(state: Arc<PirState>) -> tower_mcp::Tool {
    ToolBuilder::new("link_evidence")
        .title("Link evidence")
        .description(
            "Attach a typed evidence link (commit, PR, issue, log, dashboard, ...) to a PIR.",
        )
        .extractor_handler(
            state,
            |State(st): State<Arc<PirState>>, Json(input): Json<LinkEvidenceInput>| async move {
                let repo = match open_repo(&st) {
                    Ok(r) => r,
                    Err(e) => return err_result(e),
                };
                let kind: LinkKind = serde_json::from_value(Value::String(input.kind.clone()))
                    .unwrap_or(LinkKind::RelatedTo);
                let link = EvidenceLink {
                    uri: input.uri,
                    kind,
                    description: input.description,
                };
                match repo.link_evidence(input.pir, link) {
                    Ok(()) => ok_json(json!({ "pir": input.pir, "ok": true })),
                    Err(e) => err_result(e),
                }
            },
        )
        .build()
}

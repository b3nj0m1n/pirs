//! Core types for representing Post-Incident Reviews (PIRs).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use time::OffsetDateTime;

/// A Post-Incident Review.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pir {
    /// Sequential PIR number.
    pub number: u32,

    /// Short human-readable incident title.
    #[serde(default)]
    pub title: String,

    /// Current status of the incident.
    #[serde(default)]
    pub status: IncidentStatus,

    /// Severity classification.
    #[serde(default)]
    pub severity: IncidentSeverity,

    /// Incident type classification.
    #[serde(default, rename = "incident_type")]
    pub incident_type: IncidentType,

    /// Clear statement of the observed failure, impact, and expected behaviour.
    #[serde(default)]
    pub problem_statement: String,

    /// Best-known start time of the incident.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "ts_opt"
    )]
    pub occurred_at: Option<OffsetDateTime>,

    /// When the incident was discovered.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "ts_opt"
    )]
    pub detected_at: Option<OffsetDateTime>,

    /// When service / tests / workflow returned to acceptable state.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "ts_opt"
    )]
    pub resolved_at: Option<OffsetDateTime>,

    /// Time-to-discover (ISO-8601 duration string), derived where possible.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_to_discover: Option<String>,

    /// Time-to-resolve (ISO-8601 duration string), derived where possible.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_to_resolve: Option<String>,

    /// Total incident lifetime (ISO-8601 duration string).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_duration: Option<String>,

    /// How the incident was discovered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detection_method: Option<String>,

    /// Humans, teams, systems, or agents involved.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub people_involved: Vec<Actor>,

    /// Ordered timeline events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub timeline: Vec<TimelineEvent>,

    /// Ordered 5 Whys analysis entries.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub five_whys: Vec<WhyEntry>,

    /// Follow-up action items.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<ActionItem>,

    /// Typed evidence links.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<EvidenceLink>,

    /// Free-text impact summary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub impact: Option<String>,

    /// Concise root cause once known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_cause: Option<String>,

    /// Non-root contributing factors.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contributing_factors: Vec<String>,

    /// Positive response behaviours to preserve.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub what_went_well: Vec<String>,

    /// Process or technical gaps to improve.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub what_went_wrong: Vec<String>,

    /// Risks that did not materialize.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub where_we_got_lucky: Vec<String>,

    /// Search and grouping labels.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Confidentiality classification.
    #[serde(default)]
    pub confidentiality: Confidentiality,

    /// Optional executive summary written after review.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// Path to the file on disk (not serialized).
    #[serde(skip)]
    pub path: Option<PathBuf>,
}

impl Pir {
    /// Create a new PIR with the given number and title.
    pub fn new(number: u32, title: impl Into<String>) -> Self {
        Self {
            number,
            title: title.into(),
            status: IncidentStatus::Open,
            severity: IncidentSeverity::Low,
            incident_type: IncidentType::Development,
            problem_statement: String::new(),
            occurred_at: None,
            detected_at: None,
            resolved_at: None,
            time_to_discover: None,
            time_to_resolve: None,
            total_duration: None,
            detection_method: None,
            people_involved: Vec::new(),
            timeline: Vec::new(),
            five_whys: Vec::new(),
            actions: Vec::new(),
            links: Vec::new(),
            impact: None,
            root_cause: None,
            contributing_factors: Vec::new(),
            what_went_well: Vec::new(),
            what_went_wrong: Vec::new(),
            where_we_got_lucky: Vec::new(),
            tags: Vec::new(),
            confidentiality: Confidentiality::Internal,
            summary: None,
            path: None,
        }
    }

    /// File name for this PIR (e.g. `0001-failing-tests.md`).
    pub fn filename(&self) -> String {
        format!("{:04}-{}.md", self.number, slug(&self.title))
    }

    /// Recompute derived duration fields based on timestamps.
    pub fn recompute_durations(&mut self) {
        self.time_to_discover = match (self.occurred_at, self.detected_at) {
            (Some(a), Some(b)) if b >= a => Some(iso_duration(b - a)),
            _ => self.time_to_discover.clone(),
        };
        self.time_to_resolve = match (self.detected_at, self.resolved_at) {
            (Some(a), Some(b)) if b >= a => Some(iso_duration(b - a)),
            _ => self.time_to_resolve.clone(),
        };
        self.total_duration = match (self.occurred_at, self.resolved_at) {
            (Some(a), Some(b)) if b >= a => Some(iso_duration(b - a)),
            _ => self.total_duration.clone(),
        };
    }

    /// Allocate the next stable action ID (`ACT-NNN`).
    pub fn next_action_id(&self) -> String {
        let max = self
            .actions
            .iter()
            .filter_map(|a| {
                a.id.strip_prefix("ACT-")
                    .and_then(|n| n.parse::<u32>().ok())
            })
            .max()
            .unwrap_or(0);
        format!("ACT-{:03}", max + 1)
    }
}

// ---------------------------------------------------------------------------
// Status / severity / type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum IncidentStatus {
    #[default]
    Open,
    Investigating,
    Mitigated,
    Resolved,
    Reviewed,
    Cancelled,
    #[serde(untagged)]
    Custom(String),
}

impl std::fmt::Display for IncidentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "Open"),
            Self::Investigating => write!(f, "Investigating"),
            Self::Mitigated => write!(f, "Mitigated"),
            Self::Resolved => write!(f, "Resolved"),
            Self::Reviewed => write!(f, "Reviewed"),
            Self::Cancelled => write!(f, "Cancelled"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

impl std::str::FromStr for IncidentStatus {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "open" => Self::Open,
            "investigating" => Self::Investigating,
            "mitigated" => Self::Mitigated,
            "resolved" => Self::Resolved,
            "reviewed" => Self::Reviewed,
            "cancelled" | "canceled" => Self::Cancelled,
            _ => Self::Custom(s.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum IncidentSeverity {
    #[default]
    Low,
    Medium,
    High,
    Critical,
    #[serde(untagged)]
    Custom(String),
}

impl std::fmt::Display for IncidentSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

impl std::str::FromStr for IncidentSeverity {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "low" => Self::Low,
            "medium" | "med" => Self::Medium,
            "high" => Self::High,
            "critical" | "crit" => Self::Critical,
            _ => Self::Custom(s.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum IncidentType {
    #[default]
    Development,
    Production,
    Security,
    Process,
    #[serde(untagged)]
    Custom(String),
}

impl std::fmt::Display for IncidentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Development => write!(f, "Development"),
            Self::Production => write!(f, "Production"),
            Self::Security => write!(f, "Security"),
            Self::Process => write!(f, "Process"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

impl std::str::FromStr for IncidentType {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "development" | "dev" => Self::Development,
            "production" | "prod" => Self::Production,
            "security" | "sec" => Self::Security,
            "process" => Self::Process,
            _ => Self::Custom(s.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Confidentiality {
    Public,
    #[default]
    Internal,
    Restricted,
    #[serde(untagged)]
    Custom(String),
}

impl std::fmt::Display for Confidentiality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public => write!(f, "Public"),
            Self::Internal => write!(f, "Internal"),
            Self::Restricted => write!(f, "Restricted"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

impl std::str::FromStr for Confidentiality {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "public" => Self::Public,
            "internal" => Self::Internal,
            "restricted" => Self::Restricted,
            _ => Self::Custom(s.to_string()),
        })
    }
}

// ---------------------------------------------------------------------------
// Actor
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Actor {
    pub name: String,
    #[serde(rename = "type", default)]
    pub kind: ActorKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub directory_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
}

impl Actor {
    pub fn human(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: ActorKind::Human,
            role: None,
            email: None,
            handle: None,
            directory_id: None,
            team_id: None,
            model: None,
            provider: None,
            session_id: None,
            tool: None,
        }
    }

    pub fn agent(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: ActorKind::Agent,
            role: None,
            email: None,
            handle: None,
            directory_id: None,
            team_id: None,
            model: None,
            provider: None,
            session_id: None,
            tool: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActorKind {
    #[default]
    Human,
    Agent,
    Team,
    System,
    #[serde(untagged)]
    Custom(String),
}

impl std::fmt::Display for ActorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Human => write!(f, "human"),
            Self::Agent => write!(f, "agent"),
            Self::Team => write!(f, "team"),
            Self::System => write!(f, "system"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

impl std::str::FromStr for ActorKind {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "human" | "person" => Self::Human,
            "agent" | "llm" | "bot" => Self::Agent,
            "team" => Self::Team,
            "system" => Self::System,
            _ => Self::Custom(s.to_string()),
        })
    }
}

// ---------------------------------------------------------------------------
// Timeline / 5 whys / actions / links
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimelineEvent {
    #[serde(with = "ts_req")]
    pub at: OffsetDateTime,
    pub actor: String,
    #[serde(rename = "type")]
    pub event_type: TimelineEventType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimelineEventType {
    #[default]
    Detected,
    Investigated,
    Mitigated,
    Resolved,
    Communicated,
    Escalated,
    Note,
    #[serde(untagged)]
    Custom(String),
}

impl std::fmt::Display for TimelineEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Detected => write!(f, "detected"),
            Self::Investigated => write!(f, "investigated"),
            Self::Mitigated => write!(f, "mitigated"),
            Self::Resolved => write!(f, "resolved"),
            Self::Communicated => write!(f, "communicated"),
            Self::Escalated => write!(f, "escalated"),
            Self::Note => write!(f, "note"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

impl std::str::FromStr for TimelineEventType {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "detected" => Self::Detected,
            "investigated" => Self::Investigated,
            "mitigated" => Self::Mitigated,
            "resolved" => Self::Resolved,
            "communicated" => Self::Communicated,
            "escalated" => Self::Escalated,
            "note" => Self::Note,
            _ => Self::Custom(s.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhyEntry {
    pub question: String,
    pub answer: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionItem {
    pub id: String,
    pub description: String,
    pub owner: String,
    #[serde(default)]
    pub owner_type: ActorKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due: Option<String>,
    #[serde(default)]
    pub status: ActionStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ActionStatus {
    #[default]
    Open,
    InProgress,
    Blocked,
    Done,
    Cancelled,
    #[serde(untagged)]
    Custom(String),
}

impl std::fmt::Display for ActionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "Open"),
            Self::InProgress => write!(f, "InProgress"),
            Self::Blocked => write!(f, "Blocked"),
            Self::Done => write!(f, "Done"),
            Self::Cancelled => write!(f, "Cancelled"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

impl std::str::FromStr for ActionStatus {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "open" => Self::Open,
            "inprogress" | "in-progress" | "in_progress" => Self::InProgress,
            "blocked" => Self::Blocked,
            "done" | "closed" | "complete" | "completed" => Self::Done,
            "cancelled" | "canceled" => Self::Cancelled,
            _ => Self::Custom(s.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceLink {
    pub uri: String,
    pub kind: LinkKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum LinkKind {
    Commit,
    PullRequest,
    Issue,
    Log,
    Dashboard,
    Runbook,
    Deployment,
    TestRun,
    #[serde(rename = "ADR")]
    Adr,
    #[serde(rename = "PIR")]
    Pir,
    #[default]
    #[serde(rename = "RelatedTo")]
    RelatedTo,
    CausedBy,
    DuplicateOf,
    FollowUpTo,
    #[serde(untagged)]
    Custom(String),
}

impl std::fmt::Display for LinkKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Commit => write!(f, "Commit"),
            Self::PullRequest => write!(f, "PullRequest"),
            Self::Issue => write!(f, "Issue"),
            Self::Log => write!(f, "Log"),
            Self::Dashboard => write!(f, "Dashboard"),
            Self::Runbook => write!(f, "Runbook"),
            Self::Deployment => write!(f, "Deployment"),
            Self::TestRun => write!(f, "TestRun"),
            Self::Adr => write!(f, "ADR"),
            Self::Pir => write!(f, "PIR"),
            Self::RelatedTo => write!(f, "RelatedTo"),
            Self::CausedBy => write!(f, "CausedBy"),
            Self::DuplicateOf => write!(f, "DuplicateOf"),
            Self::FollowUpTo => write!(f, "FollowUpTo"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

impl std::str::FromStr for LinkKind {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "commit" => Self::Commit,
            "pullrequest" | "pull-request" | "pr" => Self::PullRequest,
            "issue" => Self::Issue,
            "log" => Self::Log,
            "dashboard" => Self::Dashboard,
            "runbook" => Self::Runbook,
            "deployment" => Self::Deployment,
            "testrun" | "test-run" => Self::TestRun,
            "adr" => Self::Adr,
            "pir" => Self::Pir,
            "relatedto" | "related-to" | "related" => Self::RelatedTo,
            "causedby" | "caused-by" => Self::CausedBy,
            "duplicateof" | "duplicate-of" => Self::DuplicateOf,
            "followupto" | "follow-up-to" | "followup" => Self::FollowUpTo,
            _ => Self::Custom(s.to_string()),
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a kebab-case slug for a PIR title.
pub fn slug(title: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = true;
    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            out.extend(ch.to_lowercase());
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        out.push_str("incident");
    }
    out
}

/// Format a `time::Duration` as an ISO-8601 duration (`PT...`).
pub fn iso_duration(d: time::Duration) -> String {
    let total = d.whole_seconds().max(0);
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    let mut out = String::from("PT");
    if h > 0 {
        out.push_str(&format!("{h}H"));
    }
    if m > 0 {
        out.push_str(&format!("{m}M"));
    }
    if s > 0 || (h == 0 && m == 0) {
        out.push_str(&format!("{s}S"));
    }
    out
}

mod ts_opt {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use time::OffsetDateTime;
    use time::format_description::well_known::Rfc3339;

    pub fn serialize<S: Serializer>(
        value: &Option<OffsetDateTime>,
        s: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(t) => t
                .format(&Rfc3339)
                .map_err(serde::ser::Error::custom)?
                .serialize(s),
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Option<OffsetDateTime>, D::Error> {
        let s: Option<String> = Option::deserialize(d)?;
        match s {
            Some(s) => OffsetDateTime::parse(&s, &Rfc3339)
                .map(Some)
                .map_err(serde::de::Error::custom),
            None => Ok(None),
        }
    }
}

mod ts_req {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use time::OffsetDateTime;
    use time::format_description::well_known::Rfc3339;

    pub fn serialize<S: Serializer>(value: &OffsetDateTime, s: S) -> Result<S::Ok, S::Error> {
        value
            .format(&Rfc3339)
            .map_err(serde::ser::Error::custom)?
            .serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<OffsetDateTime, D::Error> {
        let s = String::deserialize(d)?;
        OffsetDateTime::parse(&s, &Rfc3339).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_basic() {
        assert_eq!(slug("Failing Auth Tests!"), "failing-auth-tests");
        assert_eq!(slug("   "), "incident");
        assert_eq!(slug("API 500s during deploy"), "api-500s-during-deploy");
    }

    #[test]
    fn next_action_id_increments() {
        let mut p = Pir::new(1, "x");
        assert_eq!(p.next_action_id(), "ACT-001");
        p.actions.push(ActionItem {
            id: "ACT-001".into(),
            description: "x".into(),
            owner: "y".into(),
            owner_type: ActorKind::Human,
            due: None,
            status: ActionStatus::Open,
            evidence: vec![],
            notes: None,
        });
        assert_eq!(p.next_action_id(), "ACT-002");
    }

    #[test]
    fn iso_duration_format() {
        assert_eq!(iso_duration(time::Duration::seconds(0)), "PT0S");
        assert_eq!(iso_duration(time::Duration::seconds(45)), "PT45S");
        assert_eq!(iso_duration(time::Duration::minutes(2)), "PT2M");
        assert_eq!(
            iso_duration(time::Duration::seconds(3 * 3600 + 5 * 60 + 7)),
            "PT3H5M7S"
        );
    }
}

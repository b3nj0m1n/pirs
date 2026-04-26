//! Validation and lint rules for PIRs.

use crate::{ActionStatus, IncidentStatus, Pir, Repository, Result};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct Issue {
    pub number: Option<u32>,
    pub severity: IssueSeverity,
    pub message: String,
}

impl Issue {
    pub fn error(number: Option<u32>, msg: impl Into<String>) -> Self {
        Self {
            number,
            severity: IssueSeverity::Error,
            message: msg.into(),
        }
    }
    pub fn warning(number: Option<u32>, msg: impl Into<String>) -> Self {
        Self {
            number,
            severity: IssueSeverity::Warning,
            message: msg.into(),
        }
    }
    pub fn info(number: Option<u32>, msg: impl Into<String>) -> Self {
        Self {
            number,
            severity: IssueSeverity::Info,
            message: msg.into(),
        }
    }
}

#[derive(Debug, Default)]
pub struct LintReport {
    pub issues: Vec<Issue>,
}

impl LintReport {
    pub fn has_errors(&self) -> bool {
        self.issues
            .iter()
            .any(|i| i.severity == IssueSeverity::Error)
    }
    pub fn errors(&self) -> impl Iterator<Item = &Issue> {
        self.issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Error)
    }
}

/// Lint a single PIR.
pub fn lint_pir(pir: &Pir) -> Vec<Issue> {
    let mut out = Vec::new();
    if pir.title.trim().is_empty() {
        out.push(Issue::error(Some(pir.number), "title is required"));
    }
    if pir.problem_statement.trim().is_empty() {
        out.push(Issue::error(
            Some(pir.number),
            "problem_statement is required",
        ));
    }
    if let (Some(o), Some(d)) = (pir.occurred_at, pir.detected_at)
        && d < o
    {
        out.push(Issue::error(
            Some(pir.number),
            "detected_at is before occurred_at",
        ));
    }
    if let (Some(d), Some(r)) = (pir.detected_at, pir.resolved_at)
        && r < d
    {
        out.push(Issue::error(
            Some(pir.number),
            "resolved_at is before detected_at",
        ));
    }
    if matches!(pir.status, IncidentStatus::Resolved) && pir.resolved_at.is_none() {
        out.push(Issue::error(
            Some(pir.number),
            "Resolved status requires resolved_at",
        ));
    }
    for action in &pir.actions {
        if action.owner.trim().is_empty() {
            out.push(Issue::error(
                Some(pir.number),
                format!("action {} requires an owner", action.id),
            ));
        }
        if let Some(due) = &action.due
            && action.status != ActionStatus::Done
            && action.status != ActionStatus::Cancelled
        {
            // Best-effort: parse YYYY-MM-DD
            if let Ok(due_date) =
                time::Date::parse(due, &time::format_description::well_known::Iso8601::DATE)
            {
                let today = today_local();
                if due_date < today {
                    out.push(Issue::warning(
                        Some(pir.number),
                        format!("action {} is overdue (due {due})", action.id),
                    ));
                }
            }
        }
    }
    out
}

/// Lint the entire repository.
pub fn lint_repository(repo: &Repository) -> Result<LintReport> {
    let pirs = repo.list()?;
    let mut report = LintReport::default();
    let mut by_num: HashMap<u32, Vec<String>> = HashMap::new();
    for p in &pirs {
        by_num.entry(p.number).or_default().push(p.title.clone());
        report.issues.extend(lint_pir(p));
    }
    for (n, titles) in by_num {
        if titles.len() > 1 {
            report.issues.push(Issue::error(
                Some(n),
                format!("duplicate PIR number {n}: {titles:?}"),
            ));
        }
    }
    Ok(report)
}

/// Validate that a PIR is ready to move to `Reviewed`. Returns missing items.
pub fn review_gate(pir: &Pir) -> Vec<String> {
    let mut missing = Vec::new();
    if pir.problem_statement.trim().is_empty() {
        missing.push("problem_statement".into());
    }
    if pir.timeline.is_empty() {
        missing.push("at least one timeline event".into());
    }
    if pir.five_whys.is_empty() {
        missing.push("at least one 5 Whys entry".into());
    }
    if pir.actions.is_empty() {
        missing.push("at least one action item".into());
    }
    if pir.resolved_at.is_none() {
        missing.push("resolved_at".into());
    }
    missing
}

fn today_local() -> time::Date {
    time::OffsetDateTime::now_local()
        .unwrap_or_else(|_| time::OffsetDateTime::now_utc())
        .date()
}

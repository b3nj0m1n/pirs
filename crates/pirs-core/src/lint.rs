//! Validation and lint rules for PIRs.

use crate::{ActionStatus, IncidentStatus, Pir, Repository, Result};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

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

/// Blame-oriented language patterns to warn about (REQ-RPT-004).
///
/// Phrases are matched case-insensitively against PIR text fields using
/// word-boundary regexes, so substrings inside other words (e.g.
/// "blameless") do not trigger warnings. The list targets language that
/// personalises fault rather than describing systems or decisions; matches
/// are surfaced as warnings, not errors, so authors can rephrase without
/// blocking automation.
pub const BLAMEFUL_PHRASES: &[&str] = &[
    "stupid",
    "idiotic",
    "incompetent",
    "negligent",
    "lazy",
    "careless",
    "should have known",
    "should've known",
    "should have caught",
    "their fault",
    "his fault",
    "her fault",
    "to blame",
    "fault of",
    "screwed up",
    "messed up",
    "fucked up",
    "dropped the ball",
];

/// Compile each blameful phrase once into a case-insensitive regex with
/// word-boundary anchors. `\b` keeps multi-word phrases working because the
/// boundary is only checked at the start and end of the literal phrase.
fn blameful_patterns() -> &'static [(&'static str, Regex)] {
    static PATTERNS: OnceLock<Vec<(&'static str, Regex)>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        BLAMEFUL_PHRASES
            .iter()
            .map(|phrase| {
                let re = Regex::new(&format!(r"(?i)\b{}\b", regex::escape(phrase)))
                    .expect("blameful phrase pattern compiles");
                (*phrase, re)
            })
            .collect()
    })
}

/// Lint a single PIR for blame-oriented language. Returns warning issues.
///
/// Duplicate `(field_label, phrase)` pairs are emitted only once per PIR so
/// that, for example, the same phrase appearing in multiple
/// `what_went_wrong` bullets or `timeline` events does not produce a noisy
/// stream of identical warnings.
pub fn lint_language(pir: &Pir) -> Vec<Issue> {
    let mut out = Vec::new();
    let mut seen: HashSet<(String, &'static str)> = HashSet::new();
    let fields: [(&str, &str); 4] = [
        ("problem_statement", &pir.problem_statement),
        ("impact", pir.impact.as_deref().unwrap_or("")),
        ("root_cause", pir.root_cause.as_deref().unwrap_or("")),
        ("summary", pir.summary.as_deref().unwrap_or("")),
    ];
    for (label, text) in fields {
        scan_blameful(pir.number, label, text, &mut seen, &mut out);
    }
    for (i, w) in pir.five_whys.iter().enumerate() {
        scan_blameful(
            pir.number,
            &format!("five_whys[{i}].question"),
            &w.question,
            &mut seen,
            &mut out,
        );
        scan_blameful(
            pir.number,
            &format!("five_whys[{i}].answer"),
            &w.answer,
            &mut seen,
            &mut out,
        );
    }
    for ev in &pir.timeline {
        if let Some(d) = &ev.description {
            scan_blameful(pir.number, "timeline", d, &mut seen, &mut out);
        }
    }
    for v in &pir.what_went_wrong {
        scan_blameful(pir.number, "what_went_wrong", v, &mut seen, &mut out);
    }
    for v in &pir.what_went_well {
        scan_blameful(pir.number, "what_went_well", v, &mut seen, &mut out);
    }
    for v in &pir.where_we_got_lucky {
        scan_blameful(pir.number, "where_we_got_lucky", v, &mut seen, &mut out);
    }
    for v in &pir.contributing_factors {
        scan_blameful(pir.number, "contributing_factors", v, &mut seen, &mut out);
    }
    out
}

/// Lint every PIR in the repository for blame-oriented language.
pub fn lint_repository_language(repo: &Repository) -> Result<LintReport> {
    let pirs = repo.list()?;
    let mut report = LintReport::default();
    for p in &pirs {
        report.issues.extend(lint_language(p));
    }
    Ok(report)
}

fn scan_blameful(
    number: u32,
    field: &str,
    text: &str,
    seen: &mut HashSet<(String, &'static str)>,
    out: &mut Vec<Issue>,
) {
    if text.is_empty() {
        return;
    }
    for (phrase, re) in blameful_patterns() {
        if re.is_match(text) && seen.insert((field.to_string(), *phrase)) {
            out.push(Issue::warning(
                Some(number),
                format!("{field}: blame-oriented phrase \"{phrase}\""),
            ));
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pir;

    #[test]
    fn language_lint_uses_word_boundaries_and_skips_blameless() {
        let mut p = Pir::new(1, "demo");
        p.problem_statement = "We ran a blameless review of the outage.".into();
        let issues = lint_language(&p);
        assert!(
            issues.is_empty(),
            "blameless should not match (no standalone 'blame' phrase): {issues:?}"
        );
    }

    #[test]
    fn language_lint_dedups_repeated_phrase_in_same_field() {
        let mut p = Pir::new(2, "demo");
        p.what_went_wrong = vec![
            "alice was careless during deploy".into(),
            "the on-call was careless about paging".into(),
            "careless handoff between shifts".into(),
        ];
        let issues = lint_language(&p);
        let careless_hits = issues
            .iter()
            .filter(|i| i.message.contains("\"careless\""))
            .count();
        assert_eq!(careless_hits, 1, "expected one warning for the repeated phrase, got {issues:?}");
    }

    #[test]
    fn language_lint_scans_where_we_got_lucky() {
        let mut p = Pir::new(3, "demo");
        p.where_we_got_lucky = vec!["we narrowly avoided being to blame for data loss".into()];
        let issues = lint_language(&p);
        assert!(
            issues
                .iter()
                .any(|i| i.message.contains("where_we_got_lucky")
                    && i.message.contains("\"to blame\"")),
            "expected where_we_got_lucky warning, got {issues:?}"
        );
    }
}

//! Deterministic related-PIR suggestions over parsed incident records.
//!
//! The scoring helper is pure and side-effect free: callers pass an in-memory
//! slice of [`Pir`] values and receive privacy-safe suggestion metadata.

use crate::{IncidentSeverity, IncidentType, LinkKind, Pir};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const DEFAULT_LIMIT: usize = 5;
const MAX_LIMIT: usize = 20;
const DEFAULT_MIN_SCORE: u32 = 1;
const MAX_SCORE: u32 = 100;
const MAX_SHARED_TAGS: usize = 5;
const MAX_TOKEN_SCORE: u32 = 20;

/// Options controlling related-PIR suggestion output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelatedPirOptions {
    /// Maximum number of suggestions to return, capped at 20.
    pub limit: usize,
    /// Minimum score in the 0..100 range required for inclusion.
    pub min_score: u32,
}

impl RelatedPirOptions {
    /// Create options from optional MCP inputs, applying defaults and caps.
    pub fn new(limit: Option<usize>, min_score: Option<u32>) -> Self {
        Self {
            limit: limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT),
            min_score: min_score.unwrap_or(DEFAULT_MIN_SCORE).min(MAX_SCORE),
        }
    }
}

impl Default for RelatedPirOptions {
    fn default() -> Self {
        Self::new(None, None)
    }
}

/// Error returned when related-PIR suggestions cannot be computed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelatedPirError {
    /// The requested target PIR number does not exist in the provided slice.
    TargetNotFound(u32),
}

impl std::fmt::Display for RelatedPirError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TargetNotFound(number) => write!(f, "PIR {number:04} not found"),
        }
    }
}

impl std::error::Error for RelatedPirError {}

/// Non-secret explanation of why a PIR was suggested.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelatedPirSignals {
    /// Up to five shared tag values, sorted alphabetically.
    pub shared_tags: Vec<String>,
    /// Full number of shared tags, which may be higher than `shared_tags.len()`.
    pub shared_tag_count: usize,
    /// Number of shared normalized text tokens; token values are never returned.
    pub shared_token_count: usize,
    pub same_incident_type: bool,
    pub same_severity: bool,
    pub has_pir_link: bool,
}

/// Privacy-safe related-PIR suggestion metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelatedPirSuggestion {
    pub number: u32,
    pub title: String,
    pub status: String,
    pub severity: String,
    pub incident_type: String,
    pub tags: Vec<String>,
    /// Bounded score in the 0..100 range. Higher means more related.
    pub score: u32,
    pub signals: RelatedPirSignals,
}

/// Suggest PIRs related to the target PIR number.
///
/// Results are deterministic for a given input slice: this function sorts PIRs
/// by number before scoring, then orders suggestions by score descending and
/// PIR number ascending. Text matching uses ASCII lowercase tokens split on
/// non-alphanumeric characters, ignores tokens shorter than three characters,
/// and returns token counts rather than token values.
pub fn suggest_related_pirs(
    pirs: &[Pir],
    target_number: u32,
    options: RelatedPirOptions,
) -> Result<Vec<RelatedPirSuggestion>, RelatedPirError> {
    let mut ordered: Vec<&Pir> = pirs.iter().collect();
    ordered.sort_by_key(|pir| pir.number);

    let target = ordered
        .iter()
        .copied()
        .find(|pir| pir.number == target_number)
        .ok_or(RelatedPirError::TargetNotFound(target_number))?;
    let target_tokens = pir_tokens(target);
    let target_links = linked_pir_numbers(target);

    let mut suggestions = Vec::new();
    for candidate in ordered {
        if candidate.number == target_number {
            continue;
        }
        let candidate_tokens = pir_tokens(candidate);
        let candidate_links = linked_pir_numbers(candidate);
        let (score, signals) = score_candidate(
            target,
            candidate,
            &target_tokens,
            &candidate_tokens,
            &target_links,
            &candidate_links,
        );
        if score < options.min_score {
            continue;
        }
        suggestions.push(RelatedPirSuggestion {
            number: candidate.number,
            title: candidate.title.clone(),
            status: candidate.status.to_string(),
            severity: candidate.severity.to_string(),
            incident_type: candidate.incident_type.to_string(),
            tags: candidate.tags.clone(),
            score,
            signals,
        });
    }

    suggestions.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.number.cmp(&b.number)));
    suggestions.truncate(options.limit);
    Ok(suggestions)
}

fn score_candidate(
    target: &Pir,
    candidate: &Pir,
    target_tokens: &BTreeSet<String>,
    candidate_tokens: &BTreeSet<String>,
    target_links: &BTreeSet<u32>,
    candidate_links: &BTreeSet<u32>,
) -> (u32, RelatedPirSignals) {
    let target_tags: BTreeSet<&str> = target.tags.iter().map(String::as_str).collect();
    let candidate_tags: BTreeSet<&str> = candidate.tags.iter().map(String::as_str).collect();
    let shared_tag_values: Vec<String> = target_tags
        .intersection(&candidate_tags)
        .map(|tag| (*tag).to_string())
        .collect();
    let shared_tag_count = shared_tag_values.len();
    let shared_tags = shared_tag_values
        .iter()
        .take(MAX_SHARED_TAGS)
        .cloned()
        .collect();
    let shared_token_count = target_tokens.intersection(candidate_tokens).count();
    let same_incident_type = same_type(&target.incident_type, &candidate.incident_type);
    let same_severity = same_severity(&target.severity, &candidate.severity);
    let has_pir_link =
        target_links.contains(&candidate.number) || candidate_links.contains(&target.number);

    let content_score = shared_tag_count.min(MAX_SHARED_TAGS) as u32 * 8
        + (shared_token_count as u32).min(MAX_TOKEN_SCORE)
        + if has_pir_link { 40 } else { 0 };
    let score = if content_score == 0 {
        0
    } else {
        (content_score
            + if same_incident_type { 10 } else { 0 }
            + if same_severity { 5 } else { 0 })
        .min(MAX_SCORE)
    };

    (
        score,
        RelatedPirSignals {
            shared_tags,
            shared_tag_count,
            shared_token_count,
            same_incident_type,
            same_severity,
            has_pir_link,
        },
    )
}

fn same_type(left: &IncidentType, right: &IncidentType) -> bool {
    left == right
}

fn same_severity(left: &IncidentSeverity, right: &IncidentSeverity) -> bool {
    left == right
}

fn pir_tokens(pir: &Pir) -> BTreeSet<String> {
    let mut tokens = BTreeSet::new();
    collect_tokens(&pir.title, &mut tokens);
    collect_tokens(&pir.problem_statement, &mut tokens);
    if let Some(root_cause) = &pir.root_cause {
        collect_tokens(root_cause, &mut tokens);
    }
    for factor in &pir.contributing_factors {
        collect_tokens(factor, &mut tokens);
    }
    for event in &pir.timeline {
        if let Some(description) = &event.description {
            collect_tokens(description, &mut tokens);
        }
    }
    for why in &pir.five_whys {
        collect_tokens(&why.question, &mut tokens);
        collect_tokens(&why.answer, &mut tokens);
    }
    for action in &pir.actions {
        collect_tokens(&action.description, &mut tokens);
    }
    tokens
}

fn collect_tokens(text: &str, tokens: &mut BTreeSet<String>) {
    for token in text
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .map(str::to_ascii_lowercase)
        .filter(|token| token.len() >= 3)
    {
        tokens.insert(token);
    }
}

fn linked_pir_numbers(pir: &Pir) -> BTreeSet<u32> {
    let mut numbers = BTreeSet::new();
    for link in &pir.links {
        if !is_pir_relationship(&link.kind) {
            continue;
        }
        numbers.extend(extract_numbers(&link.uri));
    }
    numbers
}

fn is_pir_relationship(kind: &LinkKind) -> bool {
    matches!(
        kind,
        LinkKind::Pir
            | LinkKind::RelatedTo
            | LinkKind::CausedBy
            | LinkKind::DuplicateOf
            | LinkKind::FollowUpTo
    )
}

fn extract_numbers(text: &str) -> Vec<u32> {
    let mut numbers = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_ascii_digit() {
            current.push(ch);
        } else if !current.is_empty() {
            if let Ok(number) = current.parse::<u32>() {
                numbers.push(number);
            }
            current.clear();
        }
    }
    if current.is_empty() {
        return numbers;
    }
    if let Ok(number) = current.parse::<u32>() {
        numbers.push(number);
    }
    numbers
}

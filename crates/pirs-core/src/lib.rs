//! # pirs-core
//!
//! Core library for managing Post-Incident Reviews (PIRs).
//!
//! PIRs are stored as Markdown files with structured YAML frontmatter under
//! `doc/pir/` by default. The library provides parsing, repository CRUD, lint
//! and review-gate validation, templating, and JSON-PIR export.

mod config;
mod error;
pub mod export;
pub mod lint;
pub mod metrics;
mod parse;
pub mod related;
pub mod report;
mod repository;
pub mod template;
mod types;

pub use config::{
    CONFIG_FILE, Config, ConfigSource, DEFAULT_PIR_DIR, DiscoveredConfig, ENV_PIR_DIRECTORY,
    ENV_PIRS_CONFIG, LEGACY_CONFIG_FILE, McpConfig, PrivacyConfig, TemplateConfig, discover,
};
pub use error::{Error, Result};
pub use lint::{
    BLAMEFUL_PHRASES, Issue, IssueSeverity, LintReport, lint_language, lint_pir, lint_repository,
    lint_repository_language, review_gate,
};
pub use metrics::{DurationStats, IncidentMetrics, TagCount, compute_metrics, render_metrics_text};
pub use parse::Parser;
pub use related::{
    RelatedPirError, RelatedPirOptions, RelatedPirSignals, RelatedPirSuggestion,
    suggest_related_pirs,
};
pub use report::{render_action_register, render_pir_report};
pub use repository::Repository;
pub use types::{
    ActionItem, ActionStatus, Actor, ActorKind, Confidentiality, EvidenceLink, IncidentSeverity,
    IncidentStatus, IncidentType, LinkKind, Pir, TimelineEvent, TimelineEventType, WhyEntry, slug,
};

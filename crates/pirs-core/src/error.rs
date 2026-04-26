//! Error types for `pirs-core`.

use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("PIR repository not found; run `pirs init` to create one")]
    PirDirNotFound,

    #[error("PIR repository already exists: {0}")]
    PirDirExists(PathBuf),

    #[error("PIR not found: {0}")]
    PirNotFound(String),

    #[error("Multiple PIRs match '{query}': {matches:?}")]
    AmbiguousPir { query: String, matches: Vec<String> },

    #[error("Invalid PIR number: {0}")]
    InvalidNumber(String),

    #[error("Invalid PIR format in {path}: {reason}")]
    InvalidFormat { path: PathBuf, reason: String },

    #[error("Missing required field '{field}' in {path}")]
    MissingField { path: PathBuf, field: String },

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Invalid timestamp for {field}: {value}")]
    InvalidTimestamp { field: String, value: String },

    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    #[error("Template error: {0}")]
    TemplateError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

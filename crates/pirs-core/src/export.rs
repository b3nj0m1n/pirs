//! JSON-PIR v1 export / import.

use crate::{Error, Pir, PrivacyConfig, Repository, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

pub const JSON_PIR_VERSION: &str = "1";
pub const JSON_PIR_SCHEMA: &str =
    "https://raw.githubusercontent.com/b3nj0m1n/pirs/main/schema/json-pir/v1.json";
pub const REDACTED_VALUE: &str = "[REDACTED]";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInfo {
    pub root: String,
    pub pir_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonPirSingle {
    pub schema: String,
    pub version: String,
    pub pir: Pir,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonPirBulkExport {
    pub schema: String,
    pub version: String,
    pub tool: ToolInfo,
    #[serde(with = "ts")]
    pub generated_at: time::OffsetDateTime,
    pub repository: RepositoryInfo,
    pub pirs: Vec<Pir>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonPirDocument {
    Single(Box<JsonPirSingle>),
    Bulk(Box<JsonPirBulkExport>),
}

pub fn export_pir(pir: Pir) -> JsonPirSingle {
    JsonPirSingle {
        schema: JSON_PIR_SCHEMA.into(),
        version: JSON_PIR_VERSION.into(),
        pir,
    }
}

pub fn export_repository(repo: &Repository, tool_version: &str) -> Result<JsonPirBulkExport> {
    let pirs = repo.list()?;
    Ok(JsonPirBulkExport {
        schema: JSON_PIR_SCHEMA.into(),
        version: JSON_PIR_VERSION.into(),
        tool: ToolInfo {
            name: "pirs".into(),
            version: tool_version.into(),
        },
        generated_at: time::OffsetDateTime::now_utc(),
        repository: RepositoryInfo {
            root: repo.root().display().to_string(),
            pir_dir: repo.config().pir_dir.display().to_string(),
        },
        pirs,
    })
}

pub fn parse_json_pirs(input: &str) -> Result<Vec<Pir>> {
    let document: JsonPirDocument = serde_json::from_str(input).map_err(|err| {
        Error::Validation(format!(
            "invalid JSON-PIR input at line {}, column {}",
            err.line(),
            err.column()
        ))
    })?;
    let mut pirs = match document {
        JsonPirDocument::Single(single) => {
            let single = *single;
            ensure_version(&single.version)?;
            vec![single.pir]
        }
        JsonPirDocument::Bulk(bulk) => {
            let bulk = *bulk;
            ensure_version(&bulk.version)?;
            bulk.pirs
        }
    };
    validate_import_numbers(&pirs)?;
    for pir in &mut pirs {
        pir.path = None;
        pir.recompute_durations();
    }
    Ok(pirs)
}

pub fn redact_json_value(value: &mut Value, privacy: &PrivacyConfig) -> Result<()> {
    let sensitive_fields: HashSet<String> = privacy
        .sensitive_fields
        .iter()
        .map(|field| field.to_ascii_lowercase())
        .collect();
    let patterns = compile_redaction_patterns(&privacy.redaction_patterns)?;
    redact_value(value, &sensitive_fields, &patterns);
    Ok(())
}

fn ensure_version(version: &str) -> Result<()> {
    if version == JSON_PIR_VERSION {
        return Ok(());
    }
    Err(Error::Validation(format!(
        "unsupported JSON-PIR version `{version}`; expected `{JSON_PIR_VERSION}`"
    )))
}

fn validate_import_numbers(pirs: &[Pir]) -> Result<()> {
    let mut seen = HashSet::new();
    for pir in pirs {
        if pir.number == 0 {
            return Err(Error::Validation(
                "imported PIR number must be greater than zero".into(),
            ));
        }
        if !seen.insert(pir.number) {
            return Err(Error::Validation(format!(
                "input contains duplicate PIR number {:04}",
                pir.number
            )));
        }
    }
    Ok(())
}

fn compile_redaction_patterns(patterns: &[String]) -> Result<Vec<Regex>> {
    patterns
        .iter()
        .map(|pattern| {
            Regex::new(pattern).map_err(|err| {
                Error::ConfigError(format!("invalid redaction pattern `{pattern}`: {err}"))
            })
        })
        .collect()
}

fn redact_value(value: &mut Value, sensitive_fields: &HashSet<String>, patterns: &[Regex]) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                if sensitive_fields.contains(&key.to_ascii_lowercase()) {
                    mask_value(child);
                } else {
                    redact_value(child, sensitive_fields, patterns);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                redact_value(item, sensitive_fields, patterns);
            }
        }
        Value::String(text) => {
            for pattern in patterns {
                *text = pattern.replace_all(text, REDACTED_VALUE).to_string();
            }
        }
        _ => {}
    }
}

fn mask_value(value: &mut Value) {
    match value {
        Value::String(text) => *text = REDACTED_VALUE.into(),
        Value::Array(items) => {
            for item in items {
                mask_value(item);
            }
        }
        Value::Object(map) => {
            for child in map.values_mut() {
                mask_value(child);
            }
        }
        _ => *value = Value::String(REDACTED_VALUE.into()),
    }
}

mod ts {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use time::OffsetDateTime;
    use time::format_description::well_known::Rfc3339;

    pub fn serialize<S: Serializer>(v: &OffsetDateTime, s: S) -> Result<S::Ok, S::Error> {
        v.format(&Rfc3339)
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
    use serde_json::json;

    #[test]
    fn parse_json_pirs_rejects_unsupported_version() {
        let raw = json!({
            "schema": JSON_PIR_SCHEMA,
            "version": "2",
            "pir": {
                "number": 1,
                "title": "x",
                "status": "Open",
                "severity": "Low",
                "incident_type": "Development",
                "problem_statement": "x"
            }
        });

        let err = parse_json_pirs(&raw.to_string()).unwrap_err();
        assert!(err.to_string().contains("unsupported JSON-PIR version"));
    }

    #[test]
    fn parse_json_pirs_error_does_not_echo_input() {
        let raw = r#"{"problem_statement":"token=secret123", "#;

        let err = parse_json_pirs(raw).unwrap_err();
        let message = err.to_string();

        assert!(message.contains("invalid JSON-PIR input"));
        assert!(!message.contains("token=secret123"));
    }

    #[test]
    fn parse_json_pirs_rejects_duplicate_numbers() {
        let raw = json!({
            "schema": JSON_PIR_SCHEMA,
            "version": JSON_PIR_VERSION,
            "tool": { "name": "pirs", "version": "0.1.0" },
            "generated_at": "2026-04-26T00:00:00Z",
            "repository": { "root": ".", "pir_dir": "doc/pir" },
            "pirs": [
                {
                    "number": 1,
                    "title": "a",
                    "status": "Open",
                    "severity": "Low",
                    "incident_type": "Development",
                    "problem_statement": "a"
                },
                {
                    "number": 1,
                    "title": "b",
                    "status": "Open",
                    "severity": "Low",
                    "incident_type": "Development",
                    "problem_statement": "b"
                }
            ]
        });

        let err = parse_json_pirs(&raw.to_string()).unwrap_err();
        assert!(err.to_string().contains("duplicate PIR number 0001"));
    }

    #[test]
    fn redact_json_value_masks_patterns_and_sensitive_fields() {
        let mut value = json!({
            "pir": {
                "problem_statement": "captured token=abc123 in output",
                "summary": "private summary"
            }
        });
        let privacy = PrivacyConfig {
            redaction_patterns: vec!["token=[A-Za-z0-9_-]+".into()],
            sensitive_fields: vec!["summary".into()],
        };

        redact_json_value(&mut value, &privacy).unwrap();

        assert_eq!(
            value["pir"]["problem_statement"],
            "captured [REDACTED] in output"
        );
        assert_eq!(value["pir"]["summary"], REDACTED_VALUE);
    }
}

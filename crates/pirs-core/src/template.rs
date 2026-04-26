//! Built-in PIR templates.

use crate::{Error, IncidentType, Pir, Result};
use minijinja::{Environment, context};

/// Render a Markdown body for a PIR using the appropriate built-in template.
pub fn render(pir: &Pir, variant: &str) -> Result<String> {
    let body = match variant.to_lowercase().as_str() {
        "minimal" => MINIMAL,
        "production" => PRODUCTION,
        "security" => SECURITY,
        "process" => PROCESS,
        "development" | "" => DEVELOPMENT,
        other => return Err(Error::TemplateNotFound(other.into())),
    };
    let mut env = Environment::new();
    env.add_template("pir", body)
        .map_err(|e| Error::TemplateError(e.to_string()))?;
    let tmpl = env
        .get_template("pir")
        .map_err(|e| Error::TemplateError(e.to_string()))?;
    tmpl.render(context! {
        number => pir.number,
        title => pir.title,
        problem_statement => pir.problem_statement,
        severity => pir.severity.to_string(),
        incident_type => pir.incident_type.to_string(),
    })
    .map_err(|e| Error::TemplateError(e.to_string()))
}

/// Pick a default template variant for a given incident type.
pub fn default_variant_for(t: &IncidentType) -> &'static str {
    match t {
        IncidentType::Production => "production",
        IncidentType::Security => "security",
        IncidentType::Process => "process",
        _ => "development",
    }
}

/// Names of built-in templates.
pub const BUILTIN: &[&str] = &["development", "production", "security", "process", "minimal"];

const DEVELOPMENT: &str = include_str!("../templates/development.md");
const PRODUCTION: &str = include_str!("../templates/production.md");
const SECURITY: &str = include_str!("../templates/security.md");
const PROCESS: &str = include_str!("../templates/process.md");
const MINIMAL: &str = include_str!("../templates/minimal.md");

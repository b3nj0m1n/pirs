//! JSON-PIR v1 export / import.

use crate::{Pir, Repository, Result};
use serde::{Deserialize, Serialize};

pub const JSON_PIR_VERSION: &str = "1";
pub const JSON_PIR_SCHEMA: &str = "https://example.invalid/schema/json-pir/v1.json";

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

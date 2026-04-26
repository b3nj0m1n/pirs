use anyhow::{Context, Result};
use pirs_core::{Repository, TimelineEvent, TimelineEventType};
use std::path::Path;
use std::str::FromStr;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

pub fn add(
    cwd: &Path,
    pir: u32,
    at: Option<String>,
    actor: String,
    event_type: String,
    message: String,
) -> Result<()> {
    let repo = Repository::open(cwd).context("PIR repository not found; run `pirs init`")?;
    let at = match at {
        Some(s) => OffsetDateTime::parse(&s, &Rfc3339)
            .with_context(|| format!("invalid timestamp: {s}"))?,
        None => OffsetDateTime::now_utc(),
    };
    repo.append_timeline(
        pir,
        TimelineEvent {
            at,
            actor,
            event_type: TimelineEventType::from_str(&event_type).unwrap(),
            description: Some(message),
        },
    )?;
    println!("PIR {pir:04}: timeline event added");
    Ok(())
}

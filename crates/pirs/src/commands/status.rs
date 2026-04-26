use anyhow::{Context, Result};
use pirs_core::{IncidentStatus, Repository};
use std::path::Path;
use std::str::FromStr;
use time::OffsetDateTime;

pub fn run(cwd: &Path, pir: u32, status: &str, now: bool, reason: Option<String>) -> Result<()> {
    let repo = Repository::open(cwd).context("PIR repository not found; run `pirs init`")?;
    let new_status = IncidentStatus::from_str(status).unwrap();
    let now_ts = if now { Some(OffsetDateTime::now_utc()) } else { None };
    repo.update_status(pir, new_status.clone(), now_ts, reason)?;
    println!("PIR {pir:04} -> {new_status}");
    Ok(())
}

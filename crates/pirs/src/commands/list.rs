use anyhow::{Context, Result};
use pirs_core::{ActionStatus, Repository};
use std::path::Path;
use std::str::FromStr;

pub struct Args<'a> {
    pub cwd: &'a Path,
    pub status: Option<String>,
    pub severity: Option<String>,
    pub incident_type: Option<String>,
    pub tag: Option<String>,
    pub has_open_actions: bool,
    pub long: bool,
    pub json: bool,
}

pub fn run(args: Args<'_>) -> Result<()> {
    let repo = Repository::open(args.cwd).context("PIR repository not found; run `pirs init`")?;
    let mut pirs = repo.list()?;

    if let Some(status) = args.status.as_deref() {
        let want = pirs_core::IncidentStatus::from_str(status).unwrap();
        pirs.retain(|p| p.status == want);
    }
    if let Some(s) = args.severity.as_deref() {
        let want = pirs_core::IncidentSeverity::from_str(s).unwrap();
        pirs.retain(|p| p.severity == want);
    }
    if let Some(s) = args.incident_type.as_deref() {
        let want = pirs_core::IncidentType::from_str(s).unwrap();
        pirs.retain(|p| p.incident_type == want);
    }
    if let Some(tag) = args.tag {
        pirs.retain(|p| p.tags.iter().any(|t| t == &tag));
    }
    if args.has_open_actions {
        pirs.retain(|p| p.actions.iter().any(|a| a.status == ActionStatus::Open));
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&pirs)?);
        return Ok(());
    }

    for p in pirs {
        if args.long {
            let detected = p
                .detected_at
                .map(|t| t.date().to_string())
                .unwrap_or_else(|| "-".into());
            let ttr = p.time_to_resolve.clone().unwrap_or_else(|| "-".into());
            let open_actions = p
                .actions
                .iter()
                .filter(|a| a.status == ActionStatus::Open)
                .count();
            println!(
                "{:>4}  {:>13}  {:>8}  {}  {}  open={}  {}",
                p.number, p.status, p.severity, detected, ttr, open_actions, p.title
            );
        } else if let Some(path) = p.path {
            println!("{}", path.display());
        }
    }
    Ok(())
}

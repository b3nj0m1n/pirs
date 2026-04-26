use anyhow::{Context, Result};
use pirs_core::{ActionItem, ActionStatus, ActorKind, Repository};
use std::path::Path;
use std::str::FromStr;

pub fn add(
    cwd: &Path,
    pir: u32,
    description: String,
    owner: String,
    owner_type: String,
    due: Option<String>,
) -> Result<()> {
    let repo = Repository::open(cwd).context("PIR repository not found; run `pirs init`")?;
    let action = ActionItem {
        id: String::new(),
        description,
        owner,
        owner_type: ActorKind::from_str(&owner_type).unwrap(),
        due,
        status: ActionStatus::Open,
        evidence: Vec::new(),
        notes: None,
    };
    let id = repo.add_action(pir, action)?;
    println!("PIR {pir:04}: action {id} added");
    Ok(())
}

pub fn close(
    cwd: &Path,
    pir: u32,
    action_id: String,
    evidence: Vec<String>,
    notes: Option<String>,
) -> Result<()> {
    let repo = Repository::open(cwd).context("PIR repository not found; run `pirs init`")?;
    repo.update_action_status(pir, &action_id, ActionStatus::Done, evidence)?;
    if let Some(notes) = notes {
        let mut p = repo.get(pir)?;
        if let Some(a) = p.actions.iter_mut().find(|a| a.id == action_id) {
            a.notes = Some(notes);
        }
        repo.save(&mut p)?;
    }
    println!("PIR {pir:04}: action {action_id} closed");
    Ok(())
}

pub fn list_all(
    cwd: &Path,
    owner: Option<String>,
    status: Option<String>,
    overdue: bool,
    json: bool,
) -> Result<()> {
    let repo = Repository::open(cwd).context("PIR repository not found; run `pirs init`")?;
    let pirs = repo.list()?;
    let want_status = status
        .as_deref()
        .map(|s| ActionStatus::from_str(s).unwrap());
    let today = today_local();
    let mut rows: Vec<(u32, ActionItem)> = Vec::new();
    for p in pirs {
        for a in p.actions.iter() {
            if let Some(o) = &owner
                && &a.owner != o
            {
                continue;
            }
            if let Some(s) = &want_status
                && &a.status != s
            {
                continue;
            }
            if overdue {
                let is_overdue = a.due.as_deref().is_some_and(|d| {
                    time::Date::parse(
                        d,
                        &time::format_description::well_known::Iso8601::DATE,
                    )
                    .map(|x| x < today)
                    .unwrap_or(false)
                }) && a.status != ActionStatus::Done
                    && a.status != ActionStatus::Cancelled;
                if !is_overdue {
                    continue;
                }
            }
            rows.push((p.number, a.clone()));
        }
    }

    if json {
        let j: Vec<_> = rows
            .iter()
            .map(|(n, a)| {
                serde_json::json!({
                    "pir": n,
                    "action": a,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&j)?);
        return Ok(());
    }

    for (n, a) in rows {
        println!(
            "PIR {n:04}  {}  [{}]  {}  (owner: {}; due: {})",
            a.id,
            a.status,
            a.description,
            a.owner,
            a.due.unwrap_or_else(|| "-".into())
        );
    }
    Ok(())
}

fn today_local() -> time::Date {
    time::OffsetDateTime::now_local()
        .unwrap_or_else(|_| time::OffsetDateTime::now_utc())
        .date()
}

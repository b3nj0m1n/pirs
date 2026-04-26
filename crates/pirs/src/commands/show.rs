use anyhow::{Context, Result};
use pirs_core::Repository;
use std::path::Path;

pub fn run(cwd: &Path, query: &str, json: bool) -> Result<()> {
    let repo = Repository::open(cwd).context("PIR repository not found; run `pirs init`")?;
    let pir = repo.find(query)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&pir)?);
        return Ok(());
    }
    println!("# {:04}. {}", pir.number, pir.title);
    println!(
        "Status: {}   Severity: {}   Type: {}",
        pir.status, pir.severity, pir.incident_type
    );
    if !pir.tags.is_empty() {
        println!("Tags: {}", pir.tags.join(", "));
    }
    println!();
    if !pir.problem_statement.is_empty() {
        println!("## Problem Statement\n\n{}\n", pir.problem_statement);
    }
    if let Some(impact) = &pir.impact {
        println!("## Impact\n\n{impact}\n");
    }
    if !pir.people_involved.is_empty() {
        println!("## People and Systems Involved");
        for actor in &pir.people_involved {
            print!("- {} ({}", actor.name, actor.kind);
            if let Some(role) = &actor.role {
                print!("; role: {role}");
            }
            println!(")");
        }
        println!();
    }
    if !pir.timeline.is_empty() {
        println!("## Timeline");
        for ev in &pir.timeline {
            println!(
                "- {} [{}] {}: {}",
                ev.at,
                ev.event_type,
                ev.actor,
                ev.description.clone().unwrap_or_default()
            );
        }
        println!();
    }
    if pir.detected_at.is_some() || pir.resolved_at.is_some() {
        println!("## Detection and Resolution Timing");
        if let Some(t) = pir.time_to_discover {
            println!("- time_to_discover: {t}");
        }
        if let Some(t) = pir.time_to_resolve {
            println!("- time_to_resolve: {t}");
        }
        if let Some(t) = pir.total_duration {
            println!("- total_duration: {t}");
        }
        println!();
    }
    if !pir.five_whys.is_empty() {
        println!("## 5 Whys");
        for (i, w) in pir.five_whys.iter().enumerate() {
            println!("{}. Q: {}\n   A: {}", i + 1, w.question, w.answer);
        }
        if let Some(rc) = &pir.root_cause {
            println!("\nRoot cause: {rc}");
        }
        println!();
    }
    if !pir.actions.is_empty() {
        println!("## Actions");
        for a in &pir.actions {
            println!(
                "- {} [{}] {} (owner: {} {}; due: {})",
                a.id,
                a.status,
                a.description,
                a.owner_type,
                a.owner,
                a.due.clone().unwrap_or_else(|| "-".into())
            );
        }
        println!();
    }
    if !pir.links.is_empty() {
        println!("## Links");
        for l in &pir.links {
            println!("- [{}] {}", l.kind, l.uri);
        }
    }
    Ok(())
}

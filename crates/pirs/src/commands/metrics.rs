use anyhow::{Context, Result};
use pirs_core::{Repository, compute_metrics, render_metrics_text};
use std::path::Path;

pub fn run(cwd: &Path, json: bool) -> Result<()> {
    let repo = Repository::open(cwd).context("PIR repository not found; run `pirs init`")?;
    let pirs = repo.list()?;
    let metrics = compute_metrics(&pirs);
    if json {
        println!("{}", serde_json::to_string_pretty(&metrics)?);
    } else {
        print!("{}", render_metrics_text(&metrics));
    }
    Ok(())
}

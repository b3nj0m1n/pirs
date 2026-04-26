use anyhow::{Context, Result};
use pirs_core::{Repository, render_action_register, render_pir_report};
use std::path::Path;

pub fn report(cwd: &Path, query: &str) -> Result<()> {
    let repo = Repository::open(cwd).context("PIR repository not found; run `pirs init`")?;
    let pir = repo.find(query)?;
    print!("{}", render_pir_report(&pir));
    Ok(())
}

pub fn actions(cwd: &Path) -> Result<()> {
    let repo = Repository::open(cwd).context("PIR repository not found; run `pirs init`")?;
    let pirs = repo.list()?;
    print!("{}", render_action_register(&pirs));
    Ok(())
}

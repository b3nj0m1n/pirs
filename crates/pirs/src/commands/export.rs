use anyhow::{Context, Result, bail};
use pirs_core::{Repository, export};
use std::path::Path;

pub fn run(cwd: &Path, format: &str, pir: Option<u32>) -> Result<()> {
    if format != "json" {
        bail!("only `json` format is supported");
    }
    let repo = Repository::open(cwd).context("PIR repository not found; run `pirs init`")?;
    if let Some(n) = pir {
        let p = repo.get(n)?;
        let single = export::export_pir(p);
        println!("{}", serde_json::to_string_pretty(&single)?);
    } else {
        let bulk = export::export_repository(&repo, env!("CARGO_PKG_VERSION"))?;
        println!("{}", serde_json::to_string_pretty(&bulk)?);
    }
    Ok(())
}

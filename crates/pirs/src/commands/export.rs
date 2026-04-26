use anyhow::{Context, Result, bail};
use pirs_core::{Repository, export};
use std::path::Path;

pub fn run(cwd: &Path, format: &str, pir: Option<u32>, redact: bool) -> Result<()> {
    if format != "json" {
        bail!("only `json` format is supported");
    }
    let repo = Repository::open(cwd).context("PIR repository not found; run `pirs init`")?;
    if let Some(n) = pir {
        let p = repo.get(n)?;
        let single = export::export_pir(p);
        let mut value = serde_json::to_value(&single)?;
        if redact {
            export::redact_json_value(&mut value, &repo.config().privacy)?;
        }
        println!("{}", serde_json::to_string_pretty(&value)?);
    } else {
        let bulk = export::export_repository(&repo, env!("CARGO_PKG_VERSION"))?;
        let mut value = serde_json::to_value(&bulk)?;
        if redact {
            export::redact_json_value(&mut value, &repo.config().privacy)?;
        }
        println!("{}", serde_json::to_string_pretty(&value)?);
    }
    Ok(())
}

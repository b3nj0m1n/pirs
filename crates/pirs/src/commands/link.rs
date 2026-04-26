use anyhow::{Context, Result};
use pirs_core::{EvidenceLink, LinkKind, Repository};
use std::path::Path;
use std::str::FromStr;

pub fn run(
    cwd: &Path,
    pir: u32,
    uri: String,
    kind: String,
    description: Option<String>,
) -> Result<()> {
    let repo = Repository::open(cwd).context("PIR repository not found; run `pirs init`")?;
    repo.link_evidence(
        pir,
        EvidenceLink {
            uri,
            kind: LinkKind::from_str(&kind).unwrap(),
            description,
        },
    )?;
    println!("PIR {pir:04}: link added");
    Ok(())
}

use anyhow::{Context, Result};
use pirs_core::{Actor, ActorKind, Repository};
use std::path::Path;
use std::str::FromStr;

pub fn add(cwd: &Path, pir: u32, name: String, kind: String, role: Option<String>) -> Result<()> {
    let repo = Repository::open(cwd).context("PIR repository not found; run `pirs init`")?;
    let mut actor = Actor {
        name: name.clone(),
        kind: ActorKind::from_str(&kind).unwrap(),
        role,
        ..Actor::human(name)
    };
    // Re-set name in case the spread overwrote it (it doesn't, but make explicit)
    actor.kind = ActorKind::from_str(&kind).unwrap();
    repo.add_actor(pir, actor)?;
    println!("PIR {pir:04}: actor added");
    Ok(())
}

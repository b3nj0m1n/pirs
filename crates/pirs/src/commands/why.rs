use anyhow::{Context, Result};
use pirs_core::{Repository, WhyEntry};
use std::path::Path;

pub fn add(cwd: &Path, number: u32, question: String, answer: String, as_root: bool) -> Result<()> {
    let repo = Repository::open(cwd).context("PIR repository not found; run `pirs init`")?;
    let answer_clone = answer.clone();
    repo.add_why(number, WhyEntry { question, answer })?;
    if as_root {
        let mut pir = repo.get(number)?;
        pir.root_cause = Some(answer_clone);
        repo.save(&mut pir)?;
    }
    println!("PIR {number:04}: 5 Whys entry added");
    Ok(())
}

use anyhow::{Context, Result};
use pirs_core::Repository;
use std::path::{Path, PathBuf};

pub fn run(root: &Path, directory: PathBuf) -> Result<()> {
    let repo = Repository::init(root, Some(directory.clone())).with_context(|| {
        format!(
            "failed to initialize PIR repository in {}",
            directory.display()
        )
    })?;
    let count = repo.list().map(|p| p.len()).unwrap_or(0);
    if count > 0 {
        println!("{} ({count} existing PIRs)", repo.pir_path().display());
    } else {
        println!("{}", repo.pir_path().display());
    }
    Ok(())
}

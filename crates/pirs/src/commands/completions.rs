//! `pirs completions` — emit shell completion scripts.
//!
//! Implements REQ-COMP-001..003 per ADR-0009.
//!
//! Default: write the completion script for `<shell>` to stdout.
//! With `--out-dir DIR`: write to `<DIR>/<canonical-filename>`, creating
//! the directory if necessary.

use anyhow::{Context, Result};
use clap_complete::{Shell, generate, generate_to};
use std::fs;
use std::io;
use std::path::PathBuf;

/// Run the `completions` subcommand.
///
/// `cli` must be the top-level [`clap::Command`] for the binary; it is
/// passed in by `main` so this module does not import the CLI's private
/// types.
pub fn run(mut cli: clap::Command, shell: Shell, out_dir: Option<PathBuf>) -> Result<()> {
    let bin = cli
        .get_bin_name()
        .map(str::to_owned)
        .unwrap_or_else(|| cli.get_name().to_owned());

    match out_dir {
        None => {
            generate(shell, &mut cli, &bin, &mut io::stdout());
        }
        Some(dir) => {
            fs::create_dir_all(&dir)
                .with_context(|| format!("failed to create out-dir {}", dir.display()))?;
            let path = generate_to(shell, &mut cli, &bin, &dir).with_context(|| {
                format!(
                    "failed to write {shell} completion to {}",
                    dir.display()
                )
            })?;
            println!("{}", path.display());
        }
    }
    Ok(())
}

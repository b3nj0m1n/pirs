use anyhow::{Context, Result, bail};
use pirs_core::{Pir, Repository, export};
use std::io::Read;
use std::path::Path;

const MAX_INPUT_BYTES: u64 = 16 * 1024 * 1024;

pub struct Args<'a> {
    pub cwd: &'a Path,
    pub format: String,
    pub input: String,
    pub dry_run: bool,
    pub overwrite: bool,
}

#[derive(Debug)]
struct PlannedImport {
    action: ImportAction,
    pir: Pir,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImportAction {
    New,
    Skip,
    Overwrite,
}

pub fn run(args: Args<'_>) -> Result<()> {
    if args.format != "json" {
        bail!("only `json` format is supported");
    }
    let repo = Repository::open(args.cwd).context("PIR repository not found; run `pirs init`")?;
    let input = read_input(&args.input)
        .with_context(|| format!("failed to read JSON-PIR input `{}`", args.input))?;
    let pirs = export::parse_json_pirs(&input).context("failed to parse JSON-PIR input")?;
    let plan = plan_imports(&repo, pirs, args.overwrite)?;

    for item in &plan {
        println!(
            "{} {:04} {}",
            item.action.label(),
            item.pir.number,
            item.pir.title
        );
    }

    let would_import = plan
        .iter()
        .filter(|item| matches!(item.action, ImportAction::New | ImportAction::Overwrite))
        .count();
    let skipped = plan
        .iter()
        .filter(|item| item.action == ImportAction::Skip)
        .count();

    if args.dry_run {
        println!("summary: imported {would_import}, skipped {skipped} (dry run: no files written)");
        return Ok(());
    }

    let mut imported = 0;
    let mut overwritten = 0;
    for item in plan {
        match item.action {
            ImportAction::New => {
                repo.create(&item.pir)?;
                imported += 1;
            }
            ImportAction::Skip => {}
            ImportAction::Overwrite => {
                repo.remove_number(item.pir.number)?;
                repo.create(&item.pir)?;
                imported += 1;
                overwritten += 1;
            }
        }
    }
    println!("summary: imported {imported}, skipped {skipped}, overwritten {overwritten}");
    Ok(())
}

fn plan_imports(repo: &Repository, pirs: Vec<Pir>, overwrite: bool) -> Result<Vec<PlannedImport>> {
    pirs.into_iter()
        .map(|pir| {
            let existing_paths = repo.paths_for_number(pir.number)?;
            let action = match (existing_paths.is_empty(), overwrite) {
                (true, _) => ImportAction::New,
                (false, false) => ImportAction::Skip,
                (false, true) => ImportAction::Overwrite,
            };
            Ok(PlannedImport { action, pir })
        })
        .collect()
}

fn read_input(path: &str) -> Result<String> {
    if path == "-" {
        let stdin = std::io::stdin();
        read_to_string_limited(stdin.lock(), "stdin", MAX_INPUT_BYTES)
    } else {
        let file = std::fs::File::open(path)?;
        read_to_string_limited(file, path, MAX_INPUT_BYTES)
    }
}

fn read_to_string_limited<R: Read>(reader: R, source: &str, max_bytes: u64) -> Result<String> {
    let mut buf = String::new();
    let mut limited = reader.take(max_bytes + 1);
    limited.read_to_string(&mut buf)?;
    if buf.len() as u64 > max_bytes {
        bail!("input from `{source}` exceeds the maximum allowed size of {max_bytes} bytes");
    }
    Ok(buf)
}

impl ImportAction {
    fn label(self) -> &'static str {
        match self {
            Self::New => "NEW",
            Self::Skip => "SKIP",
            Self::Overwrite => "OVERWRITE",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn read_to_string_limited_rejects_over_limit_without_echoing_input() {
        let err = read_to_string_limited(Cursor::new("secret"), "test", 4).unwrap_err();
        let message = err.to_string();

        assert!(message.contains("exceeds the maximum allowed size"));
        assert!(!message.contains("secret"));
    }
}

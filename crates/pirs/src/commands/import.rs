use anyhow::{Context, Result, bail};
use pirs_core::{Pir, Repository, export};
use std::io::Read;
use std::path::Path;

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
            let existing_path = repo.path_for_number(pir.number)?;
            let action = match (existing_path.is_some(), overwrite) {
                (false, _) => ImportAction::New,
                (true, false) => ImportAction::Skip,
                (true, true) => ImportAction::Overwrite,
            };
            Ok(PlannedImport { action, pir })
        })
        .collect()
}

fn read_input(path: &str) -> Result<String> {
    if path == "-" {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        Ok(buf)
    } else {
        Ok(std::fs::read_to_string(path)?)
    }
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

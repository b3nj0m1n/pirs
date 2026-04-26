use anyhow::{Context, Result, bail};
use pirs_core::{IssueSeverity, Repository, lint_repository, lint_repository_language, review_gate};
use std::path::Path;

pub fn run(
    cwd: &Path,
    warnings_as_errors: bool,
    gate: Option<u32>,
    language: bool,
) -> Result<()> {
    let repo = Repository::open(cwd).context("PIR repository not found; run `pirs init`")?;

    if let Some(n) = gate {
        let pir = repo.get(n)?;
        let missing = review_gate(&pir);
        if missing.is_empty() {
            println!("PIR {n:04} is ready for Reviewed.");
            return Ok(());
        }
        for m in &missing {
            eprintln!("missing: {m}");
        }
        bail!("PIR {n:04} is not ready for Reviewed");
    }

    let mut report = lint_repository(&repo)?;
    if language {
        let lang = lint_repository_language(&repo)?;
        report.issues.extend(lang.issues);
    }
    for issue in &report.issues {
        let label = match issue.severity {
            IssueSeverity::Error => "ERROR",
            IssueSeverity::Warning => "WARN ",
            IssueSeverity::Info => "INFO ",
        };
        let num = issue
            .number
            .map(|n| format!("{n:04}"))
            .unwrap_or_else(|| "-".into());
        println!("{label}  {num}  {}", issue.message);
    }

    let has_errors = report.has_errors()
        || (warnings_as_errors
            && report
                .issues
                .iter()
                .any(|i| matches!(i.severity, IssueSeverity::Warning)));
    if has_errors {
        bail!("doctor reported errors");
    }
    Ok(())
}

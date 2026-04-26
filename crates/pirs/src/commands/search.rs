use anyhow::{Context, Result};
use pirs_core::Repository;
use std::path::Path;

pub fn run(cwd: &Path, query: &str, case_sensitive: bool) -> Result<()> {
    let repo = Repository::open(cwd).context("PIR repository not found; run `pirs init`")?;
    let needle = if case_sensitive {
        query.to_string()
    } else {
        query.to_lowercase()
    };
    let pirs = repo.list()?;
    for p in pirs {
        let mut hay = String::new();
        hay.push_str(&p.title);
        hay.push('\n');
        hay.push_str(&p.problem_statement);
        if let Some(s) = &p.impact {
            hay.push_str(s);
        }
        if let Some(s) = &p.root_cause {
            hay.push_str(s);
        }
        for ev in &p.timeline {
            if let Some(d) = &ev.description {
                hay.push_str(d);
            }
        }
        for w in &p.five_whys {
            hay.push_str(&w.question);
            hay.push_str(&w.answer);
        }
        for a in &p.actions {
            hay.push_str(&a.description);
        }
        for l in &p.links {
            hay.push_str(&l.uri);
        }
        let hay_cmp = if case_sensitive {
            hay.clone()
        } else {
            hay.to_lowercase()
        };
        if hay_cmp.contains(&needle)
            && let Some(path) = p.path
        {
            println!("{:04}: {}\n  {}", p.number, p.title, path.display());
        }
    }
    Ok(())
}

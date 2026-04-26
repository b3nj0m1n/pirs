use anyhow::{Context, Result, bail};
use pirs_core::{
    Actor, IncidentSeverity, IncidentStatus, IncidentType, Pir, Repository, TimelineEvent,
    TimelineEventType,
};
use std::io::Read;
use std::path::Path;
use std::str::FromStr;
use time::OffsetDateTime;

pub struct Args<'a> {
    pub cwd: &'a Path,
    pub title: String,
    pub problem: Option<String>,
    pub from_file: Option<String>,
    pub incident_type: Option<String>,
    pub severity: Option<String>,
    pub agent: Option<String>,
    pub tags: Vec<String>,
    pub no_initial_event: bool,
    pub no_edit: bool,
}

pub fn run(args: Args<'_>) -> Result<()> {
    let repo = Repository::open(args.cwd).context("PIR repository not found; run `pirs init`")?;
    let number = repo.next_number()?;

    let problem = match (args.problem, args.from_file) {
        (Some(p), _) => p,
        (None, Some(path)) => read_input(&path)?,
        (None, None) => {
            // Allow empty in agent / non-interactive mode but require it before Reviewed
            if args.no_edit && args.agent.is_none() {
                bail!("problem statement is required (use --problem or --from-file)");
            }
            String::new()
        }
    };

    let incident_type = match args.incident_type.as_deref() {
        Some(s) => IncidentType::from_str(s).unwrap(),
        None => IncidentType::Development,
    };

    // REQ-NEW-005a: agent-created Development incidents default to Low severity.
    let default_severity = if args.agent.is_some() && matches!(incident_type, IncidentType::Development) {
        IncidentSeverity::Low
    } else {
        IncidentSeverity::Low
    };
    let severity = match args.severity.as_deref() {
        Some(s) => IncidentSeverity::from_str(s).unwrap(),
        None => default_severity,
    };

    let mut pir = Pir::new(number, &args.title);
    pir.problem_statement = problem;
    pir.incident_type = incident_type;
    pir.severity = severity;
    pir.tags = args.tags;
    pir.status = IncidentStatus::Open;

    let now = OffsetDateTime::now_utc();
    pir.detected_at = Some(now);

    if let Some(agent) = args.agent.clone() {
        pir.people_involved.push(Actor::agent(agent));
    }

    if !args.no_initial_event {
        let actor = pir
            .people_involved
            .first()
            .map(|a| a.name.clone())
            .unwrap_or_else(|| whoami::username());
        pir.timeline.push(TimelineEvent {
            at: now,
            actor,
            event_type: TimelineEventType::Detected,
            description: Some("incident detected".into()),
        });
    }

    pir.recompute_durations();
    let path = repo.create(&pir)?;

    if !args.no_edit && atty_stdin() {
        if let Err(e) = edit::edit_file(&path) {
            eprintln!("warning: could not open editor: {e}");
        }
    }

    println!("{}", path.display());
    Ok(())
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

fn atty_stdin() -> bool {
    use std::io::IsTerminal;
    std::io::stdin().is_terminal()
}

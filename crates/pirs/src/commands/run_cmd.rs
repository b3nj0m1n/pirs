use anyhow::{Context, Result};
use pirs_core::{
    Actor, IncidentSeverity, IncidentType, Pir, Repository, TimelineEvent, TimelineEventType,
};
use std::path::Path;
use std::process::Command;
use time::OffsetDateTime;

pub struct Args<'a> {
    pub cwd: &'a Path,
    pub on_fail: String,
    pub pir_target: Option<u32>,
    pub agent: Option<String>,
    pub always_log: bool,
    pub cmd: Vec<String>,
}

const MAX_OUTPUT_BYTES: usize = 16 * 1024;

pub fn run(args: Args<'_>) -> Result<()> {
    if args.cmd.is_empty() {
        anyhow::bail!("no command supplied");
    }
    let started = OffsetDateTime::now_utc();
    let mut command = Command::new(&args.cmd[0]);
    command.args(&args.cmd[1..]);
    let output = command
        .output()
        .with_context(|| format!("failed to run {:?}", args.cmd))?;
    let finished = OffsetDateTime::now_utc();

    let exit_code = output.status.code().unwrap_or(-1);

    let success = output.status.success();
    let should_log = !success || args.always_log;
    if should_log {
        let summary = format_summary(&args.cmd, exit_code, &output.stdout, &output.stderr);
        let repo =
            Repository::open(args.cwd).context("PIR repository not found; run `pirs init`")?;
        match (args.on_fail.as_str(), args.pir_target) {
            ("none", _) => {}
            (_, Some(target)) => append_event(&repo, target, &args, &summary, exit_code, started)?,
            ("create", None) => create_pir(&repo, &args, &summary, exit_code, started, finished)?,
            (other, _) => anyhow::bail!("unknown --on-fail value: {other}"),
        }
    }

    // forward stdout/stderr
    use std::io::Write;
    std::io::stdout().write_all(&output.stdout).ok();
    std::io::stderr().write_all(&output.stderr).ok();
    std::process::exit(exit_code);
}

fn append_event(
    repo: &Repository,
    target: u32,
    args: &Args<'_>,
    summary: &str,
    exit_code: i32,
    started: OffsetDateTime,
) -> Result<()> {
    let actor = args.agent.clone().unwrap_or_else(whoami::username);
    repo.append_timeline(
        target,
        TimelineEvent {
            at: started,
            actor,
            event_type: TimelineEventType::Note,
            description: Some(format!(
                "command failed (exit {exit_code}): {}\n{summary}",
                args.cmd.join(" ")
            )),
        },
    )?;
    Ok(())
}

fn create_pir(
    repo: &Repository,
    args: &Args<'_>,
    summary: &str,
    exit_code: i32,
    started: OffsetDateTime,
    finished: OffsetDateTime,
) -> Result<()> {
    let number = repo.next_number()?;
    let title = format!("Failing command: {}", args.cmd.join(" "));
    let mut pir = Pir::new(number, &title);
    pir.problem_statement = format!(
        "Wrapped command exited with code {exit_code}.\n\nCommand: {}\n\n{summary}",
        args.cmd.join(" ")
    );
    pir.incident_type = IncidentType::Development;
    pir.severity = IncidentSeverity::Low;
    pir.detection_method = Some("agent-command-runner".into());
    pir.occurred_at = Some(started);
    pir.detected_at = Some(finished);
    if let Some(a) = &args.agent {
        pir.people_involved.push(Actor::agent(a));
    }
    let actor_name = args.agent.clone().unwrap_or_else(whoami::username);
    pir.timeline.push(TimelineEvent {
        at: finished,
        actor: actor_name,
        event_type: TimelineEventType::Detected,
        description: Some(format!("command failed (exit {exit_code})")),
    });
    pir.recompute_durations();
    let path = repo.create(&pir)?;
    eprintln!("created PIR {number:04}: {}", path.display());
    Ok(())
}

fn format_summary(cmd: &[String], code: i32, stdout: &[u8], stderr: &[u8]) -> String {
    let stdout = truncate(stdout);
    let stderr = truncate(stderr);
    format!(
        "exit_code: {code}\ncommand: {}\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}",
        cmd.join(" ")
    )
}

fn truncate(b: &[u8]) -> String {
    let s = String::from_utf8_lossy(b);
    if s.len() <= MAX_OUTPUT_BYTES {
        s.into_owned()
    } else {
        let head = &s[..MAX_OUTPUT_BYTES];
        format!("{head}\n... [truncated]")
    }
}

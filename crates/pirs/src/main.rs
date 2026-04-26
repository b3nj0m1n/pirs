//! pirs - Post-Incident Review CLI.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod mcp;

#[derive(Parser)]
#[command(name = "pirs", version, about = "Manage Post-Incident Reviews (PIRs)")]
#[command(long_about = "\
A command-line tool for creating and managing Post-Incident Reviews (PIRs).

Designed for both humans and LLM agents. Stores PIRs as Markdown with YAML
frontmatter under doc/pir by default.

GETTING STARTED:
  pirs init                          Create a new PIR repository
  pirs new \"My incident\"             Create your first PIR
  pirs list                          View all PIRs
  pirs doctor                        Check repository health")]
struct Cli {
    /// Run from a different working directory
    #[arg(short = 'C', long = "cwd", global = true, value_name = "DIR")]
    working_dir: Option<PathBuf>,

    /// Emit machine-readable JSON output where supported
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new PIR repository
    Init {
        /// Directory to store PIRs [default: doc/pir]
        #[arg(default_value = "doc/pir")]
        directory: PathBuf,
    },

    /// Create a new PIR
    New {
        /// PIR title
        title: String,
        /// Problem statement (or use --from-file)
        #[arg(short = 'p', long)]
        problem: Option<String>,
        /// Read problem statement from file (or '-' for stdin)
        #[arg(long, value_name = "PATH")]
        from_file: Option<String>,
        /// Incident type: development|production|security|process
        #[arg(short = 't', long)]
        r#type: Option<String>,
        /// Severity: low|medium|high|critical
        #[arg(short = 's', long)]
        severity: Option<String>,
        /// Agent (LLM/automation) acting as creator
        #[arg(long, value_name = "NAME")]
        agent: Option<String>,
        /// Tags (comma-separated)
        #[arg(long, value_delimiter = ',')]
        tag: Option<Vec<String>>,
        /// Suppress the initial `detected` timeline event
        #[arg(long)]
        no_initial_event: bool,
        /// Do not open `$EDITOR` after creation
        #[arg(long)]
        no_edit: bool,
    },

    /// List PIRs
    List {
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
        /// Filter by severity
        #[arg(long)]
        severity: Option<String>,
        /// Filter by incident type
        #[arg(long, name = "type")]
        type_filter: Option<String>,
        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,
        /// Only PIRs with at least one open action
        #[arg(long)]
        has_open_actions: bool,
        /// Detailed output
        #[arg(short, long)]
        long: bool,
    },

    /// Show a single PIR
    Show {
        /// PIR number or fuzzy query
        query: String,
    },

    /// Search PIRs
    Search {
        query: String,
        #[arg(short = 'c', long)]
        case_sensitive: bool,
    },

    /// Update PIR status
    Status {
        /// PIR number
        pir: u32,
        /// New status: open|investigating|mitigated|resolved|reviewed|cancelled
        status: String,
        /// Set timestamp to now where required (resolved_at)
        #[arg(long)]
        now: bool,
        /// Cancellation reason (required for `cancelled`)
        #[arg(long)]
        reason: Option<String>,
    },

    /// Append a 5 Whys entry
    Why {
        #[command(subcommand)]
        sub: WhySub,
    },

    /// Manage action items on a PIR
    Action {
        #[command(subcommand)]
        sub: ActionSub,
    },

    /// List action items across all PIRs
    Actions {
        #[arg(long)]
        owner: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        overdue: bool,
    },

    /// Manage timeline events
    Timeline {
        #[command(subcommand)]
        sub: TimelineSub,
    },

    /// Manage people / actors
    People {
        #[command(subcommand)]
        sub: PeopleSub,
    },

    /// Add a typed evidence link
    Link {
        pir: u32,
        uri: String,
        #[arg(long, default_value = "RelatedTo")]
        kind: String,
        #[arg(long)]
        description: Option<String>,
    },

    /// Validate the repository
    Doctor {
        /// Treat warnings as errors
        #[arg(long)]
        warnings_as_errors: bool,
        /// Validate that PIR `<N>` is ready for `Reviewed`
        #[arg(long, value_name = "N")]
        review_gate: Option<u32>,
    },

    /// Export PIRs as JSON-PIR
    Export {
        /// `json`
        format: String,
        /// Specific PIR number to export
        #[arg(long)]
        pir: Option<u32>,
    },

    /// Show resolved configuration
    Config,

    /// Manage built-in templates
    Template {
        #[command(subcommand)]
        sub: TemplateSub,
    },

    /// Serve PIR tools over the Model Context Protocol
    Mcp {
        #[command(subcommand)]
        sub: McpSub,
    },

    /// Run a wrapped command and optionally create a PIR on failure
    Run {
        /// What to do on failure: `create` (default) or `none`
        #[arg(long, default_value = "create")]
        on_fail: String,
        /// Create timeline event on existing PIR `<N>` instead of a new PIR
        #[arg(long, name = "pir", value_name = "N")]
        pir_target: Option<u32>,
        /// Agent identifier for actor metadata
        #[arg(long, value_name = "NAME")]
        agent: Option<String>,
        /// Always log even when the command succeeds
        #[arg(long)]
        always_log: bool,
        /// Command and args to run
        #[arg(last = true, required = true)]
        cmd: Vec<String>,
    },
}

#[derive(Subcommand)]
enum WhySub {
    /// Add a 5 Whys entry to a PIR
    Add {
        pir: u32,
        #[arg(long)]
        question: String,
        #[arg(long)]
        answer: String,
        /// Promote this answer to root_cause
        #[arg(long)]
        as_root_cause: bool,
    },
}

#[derive(Subcommand)]
enum ActionSub {
    Add {
        pir: u32,
        #[arg(long)]
        description: String,
        #[arg(long)]
        owner: String,
        #[arg(long, default_value = "human")]
        owner_type: String,
        #[arg(long)]
        due: Option<String>,
    },
    Close {
        pir: u32,
        action_id: String,
        #[arg(long)]
        evidence: Vec<String>,
        #[arg(long)]
        notes: Option<String>,
    },
}

#[derive(Subcommand)]
enum TimelineSub {
    Add {
        pir: u32,
        #[arg(long)]
        at: Option<String>,
        #[arg(long)]
        actor: String,
        #[arg(long, name = "type", default_value = "note")]
        event_type: String,
        #[arg(long)]
        message: String,
    },
}

#[derive(Subcommand)]
enum PeopleSub {
    Add {
        pir: u32,
        #[arg(long)]
        name: String,
        #[arg(long, default_value = "human")]
        kind: String,
        #[arg(long)]
        role: Option<String>,
    },
}

#[derive(Subcommand)]
enum McpSub {
    /// Start the MCP server (stdio by default)
    Serve {
        /// Bind an HTTP transport at this address (requires `http` feature)
        #[arg(long, value_name = "ADDR")]
        http: Option<String>,
        /// Default agent identifier recorded on writes when the tool call omits one
        #[arg(long, value_name = "NAME")]
        agent: Option<String>,
    },
}

#[derive(Subcommand)]
enum TemplateSub {
    /// List built-in templates
    List,
    /// Print a built-in template body
    Show { name: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cwd = match cli.working_dir.as_ref() {
        Some(d) => d.clone(),
        None => std::env::current_dir().context("failed to read current directory")?,
    };

    match cli.command {
        Commands::Init { directory } => commands::init::run(&cwd, directory),
        Commands::New {
            title,
            problem,
            from_file,
            r#type,
            severity,
            agent,
            tag,
            no_initial_event,
            no_edit,
        } => commands::new::run(commands::new::Args {
            cwd: &cwd,
            title,
            problem,
            from_file,
            incident_type: r#type,
            severity,
            agent,
            tags: tag.unwrap_or_default(),
            no_initial_event,
            no_edit,
        }),
        Commands::List {
            status,
            severity,
            type_filter,
            tag,
            has_open_actions,
            long,
        } => commands::list::run(commands::list::Args {
            cwd: &cwd,
            status,
            severity,
            incident_type: type_filter,
            tag,
            has_open_actions,
            long,
            json: cli.json,
        }),
        Commands::Show { query } => commands::show::run(&cwd, &query, cli.json),
        Commands::Search { query, case_sensitive } => {
            commands::search::run(&cwd, &query, case_sensitive)
        }
        Commands::Status {
            pir,
            status,
            now,
            reason,
        } => commands::status::run(&cwd, pir, &status, now, reason),
        Commands::Why { sub } => match sub {
            WhySub::Add {
                pir,
                question,
                answer,
                as_root_cause,
            } => commands::why::add(&cwd, pir, question, answer, as_root_cause),
        },
        Commands::Action { sub } => match sub {
            ActionSub::Add {
                pir,
                description,
                owner,
                owner_type,
                due,
            } => commands::action::add(&cwd, pir, description, owner, owner_type, due),
            ActionSub::Close {
                pir,
                action_id,
                evidence,
                notes,
            } => commands::action::close(&cwd, pir, action_id, evidence, notes),
        },
        Commands::Actions {
            owner,
            status,
            overdue,
        } => commands::action::list_all(&cwd, owner, status, overdue, cli.json),
        Commands::Timeline { sub } => match sub {
            TimelineSub::Add {
                pir,
                at,
                actor,
                event_type,
                message,
            } => commands::timeline::add(&cwd, pir, at, actor, event_type, message),
        },
        Commands::People { sub } => match sub {
            PeopleSub::Add {
                pir,
                name,
                kind,
                role,
            } => commands::people::add(&cwd, pir, name, kind, role),
        },
        Commands::Link {
            pir,
            uri,
            kind,
            description,
        } => commands::link::run(&cwd, pir, uri, kind, description),
        Commands::Doctor {
            warnings_as_errors,
            review_gate,
        } => commands::doctor::run(&cwd, warnings_as_errors, review_gate),
        Commands::Export { format, pir } => {
            commands::export::run(&cwd, &format, pir)
        }
        Commands::Config => commands::config::run(&cwd),
        Commands::Template { sub } => match sub {
            TemplateSub::List => commands::template::list(),
            TemplateSub::Show { name } => commands::template::show(&name),
        },
        Commands::Mcp { sub } => match sub {
            McpSub::Serve { http, agent } => mcp::serve(
                mcp::PirState {
                    root: cwd.clone(),
                    agent,
                },
                http,
            ),
        },
        Commands::Run {
            on_fail,
            pir_target,
            agent,
            always_log,
            cmd,
        } => commands::run_cmd::run(commands::run_cmd::Args {
            cwd: &cwd,
            on_fail,
            pir_target,
            agent,
            always_log,
            cmd,
        }),
    }
}

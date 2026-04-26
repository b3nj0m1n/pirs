//! Repository operations for PIRs.

use crate::{
    ActionItem, ActionStatus, Actor, Config, Error, EvidenceLink, IncidentStatus, Parser, Pir,
    Result, TimelineEvent, WhyEntry, template,
};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use std::fs;
use std::path::{Path, PathBuf};
use time::OffsetDateTime;
use walkdir::WalkDir;

/// Filesystem-backed PIR repository.
#[derive(Debug)]
pub struct Repository {
    root: PathBuf,
    config: Config,
    parser: Parser,
}

impl Repository {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        let config = Config::load(&root)?;
        Ok(Self {
            root,
            config,
            parser: Parser::new(),
        })
    }

    pub fn open_or_default(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let config = Config::load_or_default(&root);
        Self {
            root,
            config,
            parser: Parser::new(),
        }
    }

    /// Initialize a new PIR repository.
    pub fn init(root: impl Into<PathBuf>, pir_dir: Option<PathBuf>) -> Result<Self> {
        let root = root.into();
        let pir_dir = pir_dir.unwrap_or_else(|| PathBuf::from(crate::config::DEFAULT_PIR_DIR));
        let pir_path = root.join(&pir_dir);
        if !pir_path.exists() {
            fs::create_dir_all(&pir_path)?;
        }
        let config = Config {
            pir_dir,
            ..Default::default()
        };
        config.save(&root)?;
        Ok(Self {
            root,
            config,
            parser: Parser::new(),
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn pir_path(&self) -> PathBuf {
        self.config.pir_path(&self.root)
    }

    /// List all PIRs sorted by number.
    pub fn list(&self) -> Result<Vec<Pir>> {
        let pir_path = self.pir_path();
        if !pir_path.exists() {
            return Err(Error::PirDirNotFound);
        }
        let mut out: Vec<Pir> = WalkDir::new(&pir_path)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().is_some_and(|ext| ext == "md")
                    && e.path()
                        .file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.chars().next().is_some_and(|c| c.is_ascii_digit()))
            })
            .filter_map(|e| self.parser.parse_file(e.path()).ok())
            .collect();
        out.sort_by_key(|p| p.number);
        Ok(out)
    }

    pub fn next_number(&self) -> Result<u32> {
        Ok(self.list()?.last().map(|p| p.number + 1).unwrap_or(1))
    }

    pub fn get(&self, number: u32) -> Result<Pir> {
        self.list()?
            .into_iter()
            .find(|p| p.number == number)
            .ok_or_else(|| Error::PirNotFound(number.to_string()))
    }

    pub fn path_for_number(&self, number: u32) -> Result<Option<PathBuf>> {
        let pir_path = self.pir_path();
        if !pir_path.exists() {
            return Err(Error::PirDirNotFound);
        }
        let prefix = format!("{number:04}-");
        for entry in fs::read_dir(pir_path)? {
            let entry = entry?;
            let path = entry.path();
            let is_markdown = path.extension().is_some_and(|ext| ext == "md");
            let has_number = path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(&prefix));
            if is_markdown && has_number {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }

    pub fn remove_number(&self, number: u32) -> Result<Option<PathBuf>> {
        let Some(path) = self.path_for_number(number)? else {
            return Ok(None);
        };
        fs::remove_file(&path)?;
        Ok(Some(path))
    }

    /// Find a PIR by number or fuzzy match on title / problem statement.
    pub fn find(&self, query: &str) -> Result<Pir> {
        if let Ok(n) = query.parse::<u32>() {
            return self.get(n);
        }
        let pirs = self.list()?;
        let matcher = SkimMatcherV2::default();
        let mut matches: Vec<_> = pirs
            .into_iter()
            .filter_map(|p| {
                let haystack = format!("{} {}", p.title, p.problem_statement);
                let score = matcher.fuzzy_match(&haystack, query)?;
                Some((p, score))
            })
            .collect();
        matches.sort_by(|a, b| b.1.cmp(&a.1));
        match matches.len() {
            0 => Err(Error::PirNotFound(query.to_string())),
            1 => Ok(matches.remove(0).0),
            _ => {
                if matches[0].1 > matches[1].1 * 2 {
                    Ok(matches.remove(0).0)
                } else {
                    Err(Error::AmbiguousPir {
                        query: query.to_string(),
                        matches: matches
                            .iter()
                            .take(5)
                            .map(|(p, _)| format!("{:04} {}", p.number, p.title))
                            .collect(),
                    })
                }
            }
        }
    }

    /// Create a new PIR file from the supplied PIR struct.
    pub fn create(&self, pir: &Pir) -> Result<PathBuf> {
        let pir_path = self.pir_path();
        fs::create_dir_all(&pir_path)?;
        let filename = pir.filename();
        let path = pir_path.join(&filename);
        if path.exists() {
            return Err(Error::PirDirExists(path));
        }
        let body = template::render(pir, template::default_variant_for(&pir.incident_type))?;
        let serialized = serialize(pir, &body)?;
        atomic_write(&path, &serialized)?;
        Ok(path)
    }

    /// Persist an updated PIR back to disk, preserving the body where possible.
    pub fn save(&self, pir: &mut Pir) -> Result<()> {
        let path = pir
            .path
            .clone()
            .ok_or_else(|| Error::Validation("PIR has no path; cannot save".into()))?;
        pir.recompute_durations();
        let existing = fs::read_to_string(&path)?;
        let body = body_after_frontmatter(&existing).unwrap_or_else(|| {
            template::render(pir, template::default_variant_for(&pir.incident_type))
                .unwrap_or_default()
        });
        let serialized = serialize(pir, &body)?;
        atomic_write(&path, &serialized)?;
        Ok(())
    }

    pub fn append_timeline(&self, number: u32, event: TimelineEvent) -> Result<()> {
        let mut pir = self.get(number)?;
        pir.timeline.push(event);
        pir.timeline.sort_by_key(|e| e.at);
        self.save(&mut pir)
    }

    pub fn add_why(&self, number: u32, entry: WhyEntry) -> Result<()> {
        let mut pir = self.get(number)?;
        pir.five_whys.push(entry);
        self.save(&mut pir)
    }

    pub fn add_action(&self, number: u32, mut action: ActionItem) -> Result<String> {
        let mut pir = self.get(number)?;
        if action.id.is_empty() {
            action.id = pir.next_action_id();
        }
        let id = action.id.clone();
        pir.actions.push(action);
        self.save(&mut pir)?;
        Ok(id)
    }

    pub fn update_action_status(
        &self,
        number: u32,
        action_id: &str,
        status: ActionStatus,
        evidence: Vec<String>,
    ) -> Result<()> {
        let mut pir = self.get(number)?;
        let action = pir
            .actions
            .iter_mut()
            .find(|a| a.id == action_id)
            .ok_or_else(|| Error::Validation(format!("action {action_id} not found")))?;
        action.status = status;
        action.evidence.extend(evidence);
        self.save(&mut pir)
    }

    pub fn link_evidence(&self, number: u32, link: EvidenceLink) -> Result<()> {
        let mut pir = self.get(number)?;
        pir.links.push(link);
        self.save(&mut pir)
    }

    pub fn add_actor(&self, number: u32, actor: Actor) -> Result<()> {
        let mut pir = self.get(number)?;
        if !pir.people_involved.iter().any(|a| a.name == actor.name) {
            pir.people_involved.push(actor);
        }
        self.save(&mut pir)
    }

    /// Update status with validation.
    pub fn update_status(
        &self,
        number: u32,
        status: IncidentStatus,
        now: Option<OffsetDateTime>,
        cancellation_reason: Option<String>,
    ) -> Result<()> {
        let mut pir = self.get(number)?;

        match &status {
            IncidentStatus::Resolved => {
                if pir.resolved_at.is_none() {
                    if let Some(t) = now {
                        pir.resolved_at = Some(t);
                    } else {
                        return Err(Error::Validation(
                            "resolving requires resolved_at; pass --now or set timestamp".into(),
                        ));
                    }
                }
                pir.timeline.push(TimelineEvent {
                    at: pir.resolved_at.unwrap(),
                    actor: "pirs".into(),
                    event_type: crate::types::TimelineEventType::Resolved,
                    description: Some("status -> Resolved".into()),
                });
            }
            IncidentStatus::Reviewed => {
                let issues = crate::lint::review_gate(&pir);
                if !issues.is_empty() {
                    return Err(Error::Validation(format!(
                        "PIR is not ready for Reviewed: {}",
                        issues.join("; ")
                    )));
                }
            }
            IncidentStatus::Cancelled => {
                if cancellation_reason.is_none() {
                    return Err(Error::Validation(
                        "cancelling requires a reason (--reason)".into(),
                    ));
                }
                pir.summary = cancellation_reason;
            }
            _ => {}
        }
        pir.status = status;
        self.save(&mut pir)
    }
}

// ---------------------------------------------------------------------------
// Serialization helpers
// ---------------------------------------------------------------------------

fn serialize(pir: &Pir, body: &str) -> Result<String> {
    let yaml = serde_yaml::to_string(pir)?;
    let body = body.trim_start_matches('\n');
    Ok(format!("---\n{yaml}---\n\n{body}\n"))
}

fn body_after_frontmatter(content: &str) -> Option<String> {
    if !content.starts_with("---\n") {
        return None;
    }
    let parts: Vec<&str> = content.splitn(3, "---\n").collect();
    if parts.len() < 3 {
        return None;
    }
    Some(parts[2].trim_start_matches('\n').to_string())
}

fn atomic_write(path: &Path, content: &str) -> Result<()> {
    let tmp = path.with_extension("md.tmp");
    fs::write(&tmp, content)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

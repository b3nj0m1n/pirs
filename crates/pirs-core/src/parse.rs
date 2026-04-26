//! PIR file parsing (YAML frontmatter + Markdown body).

use crate::{Error, Pir, Result};
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

static NUMBER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d{4})-.*\.md$").unwrap());

#[derive(Debug, Default)]
pub struct Parser {
    _private: (),
}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a PIR from a file.
    pub fn parse_file(&self, path: &Path) -> Result<Pir> {
        let content = std::fs::read_to_string(path)?;
        let mut pir = self.parse(&content).map_err(|e| match e {
            Error::InvalidFormat { reason, .. } => Error::InvalidFormat {
                path: path.to_path_buf(),
                reason,
            },
            other => other,
        })?;
        if pir.number == 0 {
            pir.number = extract_number_from_path(path)?;
        }
        if pir.title.is_empty()
            && let Some(title) = extract_h1_title(&content)
        {
            pir.title = title;
        }
        pir.path = Some(path.to_path_buf());
        pir.recompute_durations();
        Ok(pir)
    }

    /// Parse a PIR from a string. Requires YAML frontmatter.
    pub fn parse(&self, content: &str) -> Result<Pir> {
        if !content.starts_with("---\n") {
            return Err(Error::InvalidFormat {
                path: Default::default(),
                reason: "PIR file must start with YAML frontmatter".into(),
            });
        }
        let parts: Vec<&str> = content.splitn(3, "---\n").collect();
        if parts.len() < 3 {
            return Err(Error::InvalidFormat {
                path: Default::default(),
                reason: "Invalid frontmatter format".into(),
            });
        }
        let yaml = parts[1];
        let body = parts[2];

        let mut pir: Pir = serde_yaml::from_str(yaml)?;

        // Pull free-text sections from the body if not already populated.
        let sections = parse_h2_sections(body);
        if pir.problem_statement.is_empty()
            && let Some(s) = sections.get("problem statement")
        {
            pir.problem_statement = s.clone();
        }
        if pir.impact.is_none()
            && let Some(s) = sections.get("impact")
            && !s.is_empty()
        {
            pir.impact = Some(s.clone());
        }
        if pir.summary.is_none()
            && let Some(s) = sections.get("summary")
            && !s.is_empty()
        {
            pir.summary = Some(s.clone());
        }

        Ok(pir)
    }
}

/// Extract a kebab `## Heading` -> body map from markdown.
pub fn parse_h2_sections(body: &str) -> std::collections::HashMap<String, String> {
    let mut out = std::collections::HashMap::new();
    let mut current: Option<String> = None;
    let mut buf = String::new();
    for line in body.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            if let Some(name) = current.take() {
                out.insert(name, buf.trim().to_string());
                buf.clear();
            }
            current = Some(rest.trim().to_lowercase());
        } else if current.is_some() {
            buf.push_str(line);
            buf.push('\n');
        }
    }
    if let Some(name) = current {
        out.insert(name, buf.trim().to_string());
    }
    out
}

fn extract_h1_title(content: &str) -> Option<String> {
    let line = content.lines().find(|l| l.starts_with("# "))?;
    let rest = line.trim_start_matches("# ").trim();
    if rest.is_empty() {
        return None;
    }
    // strip leading "1. " number prefix if present
    if let Some((num_part, title)) = rest.split_once(". ")
        && num_part.chars().all(|c| c.is_ascii_digit())
    {
        return Some(title.to_string());
    }
    Some(rest.to_string())
}

fn extract_number_from_path(path: &Path) -> Result<u32> {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| Error::InvalidFormat {
            path: path.to_path_buf(),
            reason: "invalid filename".into(),
        })?;
    NUMBER_REGEX
        .captures(filename)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<u32>().ok())
        .ok_or_else(|| Error::InvalidFormat {
            path: path.to_path_buf(),
            reason: "filename does not start with NNNN-".into(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_pir() {
        let raw = "---\nnumber: 1\ntitle: t\nstatus: Open\nseverity: Low\nincident_type: Development\nproblem_statement: \"x\"\n---\n# 1. t\n";
        let p = Parser::new().parse(raw).unwrap();
        assert_eq!(p.number, 1);
        assert_eq!(p.problem_statement, "x");
    }

    #[test]
    fn parse_h2_sections_basic() {
        let body = "## Problem Statement\n\nfoo\n\n## Impact\n\nbar\n";
        let s = parse_h2_sections(body);
        assert_eq!(s.get("problem statement").unwrap(), "foo");
        assert_eq!(s.get("impact").unwrap(), "bar");
    }
}

//! Aggregate incident metrics across a PIR repository (REQ-RPT-003).
//!
//! Pure transformations over `Pir` slices. The CLI prints either a textual
//! summary or JSON; the MCP `get_incident_metrics` tool will reuse this.

use crate::{ActionStatus, IncidentSeverity, IncidentStatus, IncidentType, Pir};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Aggregate metrics across a set of PIRs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IncidentMetrics {
    pub total: usize,
    pub by_status: BTreeMap<String, usize>,
    pub by_severity: BTreeMap<String, usize>,
    pub by_type: BTreeMap<String, usize>,
    /// Time-to-discover statistics in seconds (mean, median).
    pub ttd_seconds: Option<DurationStats>,
    /// Time-to-resolve statistics in seconds (mean, median).
    pub ttr_seconds: Option<DurationStats>,
    /// Tag occurrences sorted by count (descending), then alphabetically.
    pub recurring_tags: Vec<TagCount>,
    pub open_actions: usize,
    pub total_actions: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct DurationStats {
    pub count: usize,
    pub mean_seconds: i64,
    pub median_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TagCount {
    pub tag: String,
    pub count: usize,
}

/// Compute aggregate metrics over the provided PIRs.
pub fn compute_metrics(pirs: &[Pir]) -> IncidentMetrics {
    let total = pirs.len();
    let mut by_status: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_severity: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_type: BTreeMap<String, usize> = BTreeMap::new();
    let mut ttd: Vec<i64> = Vec::new();
    let mut ttr: Vec<i64> = Vec::new();
    let mut tag_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut open_actions = 0usize;
    let mut total_actions = 0usize;

    for pir in pirs {
        *by_status
            .entry(status_key(&pir.status))
            .or_insert(0) += 1;
        *by_severity
            .entry(severity_key(&pir.severity))
            .or_insert(0) += 1;
        *by_type.entry(type_key(&pir.incident_type)).or_insert(0) += 1;

        if let (Some(o), Some(d)) = (pir.occurred_at, pir.detected_at) {
            let secs = (d - o).whole_seconds();
            if secs >= 0 {
                ttd.push(secs);
            }
        }
        if let (Some(d), Some(r)) = (pir.detected_at, pir.resolved_at) {
            let secs = (r - d).whole_seconds();
            if secs >= 0 {
                ttr.push(secs);
            }
        }

        for tag in &pir.tags {
            *tag_counts.entry(tag.clone()).or_insert(0) += 1;
        }

        for action in &pir.actions {
            total_actions += 1;
            if !matches!(
                action.status,
                ActionStatus::Done | ActionStatus::Cancelled
            ) {
                open_actions += 1;
            }
        }
    }

    let mut recurring_tags: Vec<TagCount> = tag_counts
        .into_iter()
        .map(|(tag, count)| TagCount { tag, count })
        .collect();
    recurring_tags.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.tag.cmp(&b.tag)));

    IncidentMetrics {
        total,
        by_status,
        by_severity,
        by_type,
        ttd_seconds: stats(&ttd),
        ttr_seconds: stats(&ttr),
        recurring_tags,
        open_actions,
        total_actions,
    }
}

/// Render metrics as a human-readable text summary.
pub fn render_metrics_text(m: &IncidentMetrics) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    let _ = writeln!(out, "Incidents: {}", m.total);
    let _ = writeln!(out, "Total actions: {}   Open: {}", m.total_actions, m.open_actions);
    let _ = writeln!(out);

    let _ = writeln!(out, "By status:");
    for (k, v) in &m.by_status {
        let _ = writeln!(out, "  {k}: {v}");
    }
    let _ = writeln!(out, "By severity:");
    for (k, v) in &m.by_severity {
        let _ = writeln!(out, "  {k}: {v}");
    }
    let _ = writeln!(out, "By type:");
    for (k, v) in &m.by_type {
        let _ = writeln!(out, "  {k}: {v}");
    }
    let _ = writeln!(out);

    let _ = writeln!(out, "Time to discover:");
    match &m.ttd_seconds {
        Some(s) => {
            let _ = writeln!(
                out,
                "  count={} mean={} median={}",
                s.count,
                fmt_seconds(s.mean_seconds),
                fmt_seconds(s.median_seconds)
            );
        }
        None => {
            let _ = writeln!(out, "  no data");
        }
    }
    let _ = writeln!(out, "Time to resolve:");
    match &m.ttr_seconds {
        Some(s) => {
            let _ = writeln!(
                out,
                "  count={} mean={} median={}",
                s.count,
                fmt_seconds(s.mean_seconds),
                fmt_seconds(s.median_seconds)
            );
        }
        None => {
            let _ = writeln!(out, "  no data");
        }
    }
    let _ = writeln!(out);

    let _ = writeln!(out, "Recurring tags:");
    if m.recurring_tags.is_empty() {
        let _ = writeln!(out, "  none");
    } else {
        for tc in m.recurring_tags.iter().take(10) {
            let _ = writeln!(out, "  {}: {}", tc.tag, tc.count);
        }
    }
    out
}

fn stats(values: &[i64]) -> Option<DurationStats> {
    if values.is_empty() {
        return None;
    }
    let count = values.len();
    let sum: i128 = values.iter().map(|v| *v as i128).sum();
    let mean = (sum / count as i128) as i64;
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let median = if count % 2 == 1 {
        sorted[count / 2]
    } else {
        let a = sorted[count / 2 - 1];
        let b = sorted[count / 2];
        ((a as i128 + b as i128) / 2) as i64
    };
    Some(DurationStats {
        count,
        mean_seconds: mean,
        median_seconds: median,
    })
}

fn status_key(s: &IncidentStatus) -> String {
    s.to_string()
}
fn severity_key(s: &IncidentSeverity) -> String {
    s.to_string()
}
fn type_key(s: &IncidentType) -> String {
    s.to_string()
}

fn fmt_seconds(s: i64) -> String {
    let total = s.max(0);
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let sec = total % 60;
    if h > 0 {
        format!("{h}h{m}m{sec}s")
    } else if m > 0 {
        format!("{m}m{sec}s")
    } else {
        format!("{sec}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ActionItem, ActionStatus, ActorKind, IncidentSeverity, Pir};
    use time::OffsetDateTime;
    use time::macros::datetime;

    fn pir(num: u32, sev: IncidentSeverity) -> Pir {
        let mut p = Pir::new(num, format!("p{num}"));
        p.severity = sev;
        p
    }

    fn at(t: OffsetDateTime) -> Option<OffsetDateTime> {
        Some(t)
    }

    #[test]
    fn empty_repo_returns_zero_total() {
        let m = compute_metrics(&[]);
        assert_eq!(m.total, 0);
        assert!(m.ttd_seconds.is_none());
        assert!(m.ttr_seconds.is_none());
        assert_eq!(m.open_actions, 0);
    }

    #[test]
    fn counts_buckets_and_actions() {
        let mut p1 = pir(1, IncidentSeverity::High);
        p1.tags = vec!["ci".into(), "flaky".into()];
        p1.actions.push(ActionItem {
            id: "ACT-001".into(),
            description: "x".into(),
            owner: "a".into(),
            owner_type: ActorKind::Human,
            due: None,
            status: ActionStatus::Open,
            evidence: vec![],
            notes: None,
        });
        let mut p2 = pir(2, IncidentSeverity::High);
        p2.tags = vec!["ci".into()];
        p2.actions.push(ActionItem {
            id: "ACT-001".into(),
            description: "y".into(),
            owner: "b".into(),
            owner_type: ActorKind::Human,
            due: None,
            status: ActionStatus::Done,
            evidence: vec![],
            notes: None,
        });

        let m = compute_metrics(&[p1, p2]);
        assert_eq!(m.total, 2);
        assert_eq!(m.by_severity.get("High").copied(), Some(2));
        assert_eq!(m.total_actions, 2);
        assert_eq!(m.open_actions, 1);
        assert_eq!(m.recurring_tags[0].tag, "ci");
        assert_eq!(m.recurring_tags[0].count, 2);
    }

    #[test]
    fn computes_ttd_and_ttr_stats() {
        let mut p1 = Pir::new(1, "p1");
        p1.occurred_at = at(datetime!(2026-01-01 00:00:00 UTC));
        p1.detected_at = at(datetime!(2026-01-01 00:01:00 UTC)); // 60s
        p1.resolved_at = at(datetime!(2026-01-01 00:11:00 UTC)); // 600s

        let mut p2 = Pir::new(2, "p2");
        p2.occurred_at = at(datetime!(2026-01-01 00:00:00 UTC));
        p2.detected_at = at(datetime!(2026-01-01 00:03:00 UTC)); // 180s
        p2.resolved_at = at(datetime!(2026-01-01 00:23:00 UTC)); // 1200s

        let m = compute_metrics(&[p1, p2]);
        let ttd = m.ttd_seconds.unwrap();
        assert_eq!(ttd.count, 2);
        assert_eq!(ttd.mean_seconds, 120);
        assert_eq!(ttd.median_seconds, 120);
        let ttr = m.ttr_seconds.unwrap();
        assert_eq!(ttr.count, 2);
        assert_eq!(ttr.mean_seconds, 900);
        assert_eq!(ttr.median_seconds, 900);
    }
}

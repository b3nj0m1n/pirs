//! Markdown report rendering for individual PIRs and cross-PIR action registers.
//!
//! Pure functions over `Pir` values; the CLI wraps these with I/O.
//! Implements REQ-RPT-001 (`generate report <PIR>`) and
//! REQ-RPT-002 (`generate actions`).

use crate::{ActionStatus, Pir};
use std::fmt::Write as _;

/// Render a single PIR as a Markdown report containing summary, problem
/// statement, impact, timeline, timing metrics, 5 Whys, actions, and
/// lessons learned (REQ-RPT-001).
pub fn render_pir_report(pir: &Pir) -> String {
    let mut out = String::new();

    let _ = writeln!(out, "# PIR-{:04}: {}", pir.number, pir.title);
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "- Status: {}\n- Severity: {}\n- Type: {}",
        pir.status, pir.severity, pir.incident_type
    );
    if !pir.tags.is_empty() {
        let _ = writeln!(out, "- Tags: {}", pir.tags.join(", "));
    }
    let _ = writeln!(out);

    if let Some(summary) = &pir.summary {
        let _ = writeln!(out, "## Summary\n\n{summary}\n");
    }

    if !pir.problem_statement.is_empty() {
        let _ = writeln!(out, "## Problem Statement\n\n{}\n", pir.problem_statement);
    }

    if let Some(impact) = &pir.impact {
        let _ = writeln!(out, "## Impact\n\n{impact}\n");
    }

    if !pir.people_involved.is_empty() {
        let _ = writeln!(out, "## People and Systems Involved\n");
        for actor in &pir.people_involved {
            let role = actor
                .role
                .as_deref()
                .map(|r| format!("; role: {r}"))
                .unwrap_or_default();
            let _ = writeln!(out, "- {} ({}{role})", actor.name, actor.kind);
        }
        let _ = writeln!(out);
    }

    if !pir.timeline.is_empty() {
        let _ = writeln!(out, "## Timeline\n");
        for ev in &pir.timeline {
            let desc = ev.description.clone().unwrap_or_default();
            let _ = writeln!(
                out,
                "- {} [{}] {}: {desc}",
                ev.at, ev.event_type, ev.actor
            );
        }
        let _ = writeln!(out);
    }

    if pir.occurred_at.is_some() || pir.detected_at.is_some() || pir.resolved_at.is_some() {
        let _ = writeln!(out, "## Detection and Resolution Timing\n");
        if let Some(t) = &pir.time_to_discover {
            let _ = writeln!(out, "- time_to_discover: {t}");
        }
        if let Some(t) = &pir.time_to_resolve {
            let _ = writeln!(out, "- time_to_resolve: {t}");
        }
        if let Some(t) = &pir.total_duration {
            let _ = writeln!(out, "- total_duration: {t}");
        }
        if let Some(m) = &pir.detection_method {
            let _ = writeln!(out, "- detection_method: {m}");
        }
        let _ = writeln!(out);
    }

    if !pir.five_whys.is_empty() {
        let _ = writeln!(out, "## 5 Whys\n");
        for (i, w) in pir.five_whys.iter().enumerate() {
            let _ = writeln!(out, "{}. Q: {}\n   A: {}", i + 1, w.question, w.answer);
        }
        if let Some(rc) = &pir.root_cause {
            let _ = writeln!(out, "\nRoot cause: {rc}");
        }
        let _ = writeln!(out);
    }

    if !pir.actions.is_empty() {
        let _ = writeln!(out, "## Actions\n");
        for a in &pir.actions {
            let due = a.due.clone().unwrap_or_else(|| "-".into());
            let _ = writeln!(
                out,
                "- {} [{}] {} (owner: {} {}; due: {due})",
                a.id, a.status, a.description, a.owner_type, a.owner
            );
        }
        let _ = writeln!(out);
    }

    let has_lessons = !pir.what_went_well.is_empty()
        || !pir.what_went_wrong.is_empty()
        || !pir.where_we_got_lucky.is_empty()
        || !pir.contributing_factors.is_empty();
    if has_lessons {
        let _ = writeln!(out, "## Lessons Learned\n");
        if !pir.what_went_well.is_empty() {
            let _ = writeln!(out, "### What went well\n");
            for x in &pir.what_went_well {
                let _ = writeln!(out, "- {x}");
            }
            let _ = writeln!(out);
        }
        if !pir.what_went_wrong.is_empty() {
            let _ = writeln!(out, "### What went wrong\n");
            for x in &pir.what_went_wrong {
                let _ = writeln!(out, "- {x}");
            }
            let _ = writeln!(out);
        }
        if !pir.where_we_got_lucky.is_empty() {
            let _ = writeln!(out, "### Where we got lucky\n");
            for x in &pir.where_we_got_lucky {
                let _ = writeln!(out, "- {x}");
            }
            let _ = writeln!(out);
        }
        if !pir.contributing_factors.is_empty() {
            let _ = writeln!(out, "### Contributing factors\n");
            for x in &pir.contributing_factors {
                let _ = writeln!(out, "- {x}");
            }
            let _ = writeln!(out);
        }
    }

    if !pir.links.is_empty() {
        let _ = writeln!(out, "## Links\n");
        for l in &pir.links {
            let desc = l
                .description
                .as_deref()
                .map(|d| format!(" — {d}"))
                .unwrap_or_default();
            let _ = writeln!(out, "- [{}] {}{desc}", l.kind, l.uri);
        }
        let _ = writeln!(out);
    }

    out
}

/// Render a cross-PIR action register suitable for status review (REQ-RPT-002).
///
/// Open actions first (sorted by due date, undated last), then closed actions.
pub fn render_action_register(pirs: &[Pir]) -> String {
    let mut rows: Vec<RegisterRow> = Vec::new();
    for pir in pirs {
        for action in &pir.actions {
            rows.push(RegisterRow {
                pir_number: pir.number,
                pir_title: pir.title.clone(),
                action: action.clone(),
            });
        }
    }

    rows.sort_by(|a, b| {
        let a_open = !matches!(
            a.action.status,
            ActionStatus::Done | ActionStatus::Cancelled
        );
        let b_open = !matches!(
            b.action.status,
            ActionStatus::Done | ActionStatus::Cancelled
        );
        b_open
            .cmp(&a_open)
            .then_with(|| due_sort_key(&a.action.due).cmp(&due_sort_key(&b.action.due)))
            .then_with(|| a.pir_number.cmp(&b.pir_number))
            .then_with(|| a.action.id.cmp(&b.action.id))
    });

    let mut out = String::new();
    let _ = writeln!(out, "# Action Register\n");
    let total = rows.len();
    let open = rows
        .iter()
        .filter(|r| {
            !matches!(
                r.action.status,
                ActionStatus::Done | ActionStatus::Cancelled
            )
        })
        .count();
    let _ = writeln!(out, "Total actions: {total}   Open: {open}\n");

    if rows.is_empty() {
        let _ = writeln!(out, "_No actions recorded._");
        return out;
    }

    let _ = writeln!(
        out,
        "| PIR | Action | Status | Owner | Due | Description |"
    );
    let _ = writeln!(
        out,
        "| --- | --- | --- | --- | --- | --- |"
    );
    for r in &rows {
        let due = r.action.due.clone().unwrap_or_else(|| "-".into());
        let owner = format!("{} {}", r.action.owner_type, r.action.owner);
        let desc = sanitize_cell(&r.action.description);
        let title = sanitize_cell(&r.pir_title);
        let _ = writeln!(
            out,
            "| {:04} {title} | {} | {} | {owner} | {due} | {desc} |",
            r.pir_number, r.action.id, r.action.status,
        );
    }
    out
}

struct RegisterRow {
    pir_number: u32,
    pir_title: String,
    action: crate::ActionItem,
}

fn due_sort_key(due: &Option<String>) -> (u8, String) {
    match due {
        Some(d) if !d.is_empty() => (0, d.clone()),
        _ => (1, String::new()),
    }
}

fn sanitize_cell(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ActionItem, ActionStatus, ActorKind, Pir};

    fn pir_with_action(num: u32, status: ActionStatus, due: Option<&str>) -> Pir {
        let mut pir = Pir::new(num, format!("Incident {num}"));
        pir.actions.push(ActionItem {
            id: "ACT-001".into(),
            description: "do the thing".into(),
            owner: "alice".into(),
            owner_type: ActorKind::Human,
            due: due.map(String::from),
            status,
            evidence: vec![],
            notes: None,
        });
        pir
    }

    #[test]
    fn report_includes_required_sections() {
        let mut pir = Pir::new(1, "Failing tests");
        pir.problem_statement = "tests broke".into();
        pir.impact = Some("CI red".into());
        let out = render_pir_report(&pir);
        assert!(out.contains("# PIR-0001: Failing tests"));
        assert!(out.contains("## Problem Statement"));
        assert!(out.contains("## Impact"));
    }

    #[test]
    fn register_orders_open_before_closed() {
        let pirs = vec![
            pir_with_action(1, ActionStatus::Done, Some("2026-01-01")),
            pir_with_action(2, ActionStatus::Open, Some("2026-12-01")),
        ];
        let out = render_action_register(&pirs);
        let open_idx = out.find("0002").unwrap();
        let done_idx = out.find("0001").unwrap();
        assert!(open_idx < done_idx);
    }

    #[test]
    fn register_handles_empty() {
        let out = render_action_register(&[]);
        assert!(out.contains("No actions recorded"));
    }
}

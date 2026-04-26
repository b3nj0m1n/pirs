use pirs_core::{
    IncidentSeverity, IncidentType, Pir, RelatedPirError, RelatedPirOptions, suggest_related_pirs,
};
use serde_json::Value;

fn pir(number: u32, title: &str, problem: &str, tags: &[&str]) -> Pir {
    let mut pir = Pir::new(number, title);
    pir.problem_statement = problem.to_string();
    pir.tags = tags.iter().map(|tag| tag.to_string()).collect();
    pir
}

#[test]
fn suggest_related_pirs_orders_by_score_then_number_and_excludes_target() {
    let mut target = pir(
        1,
        "MCP metrics omitted",
        "Agent cannot retrieve incident metrics over MCP",
        &["mcp", "metrics"],
    );
    target.severity = IncidentSeverity::High;
    target.incident_type = IncidentType::Development;

    let mut strongest = pir(
        2,
        "MCP incident metrics missing",
        "MCP client needs incident metrics for agent workflow",
        &["mcp", "metrics"],
    );
    strongest.severity = IncidentSeverity::High;
    strongest.incident_type = IncidentType::Development;

    let mut related = pir(
        3,
        "MCP related search unclear",
        "Agent needs related incident suggestions",
        &["mcp"],
    );
    related.incident_type = IncidentType::Development;

    let unrelated = pir(
        4,
        "Template typo",
        "Markdown heading was misspelled",
        &["docs"],
    );

    let suggestions = suggest_related_pirs(
        &[target, unrelated, related, strongest],
        1,
        RelatedPirOptions::new(Some(5), Some(1)),
    )
    .expect("related suggestions");

    let numbers: Vec<u32> = suggestions
        .iter()
        .map(|suggestion| suggestion.number)
        .collect();
    assert_eq!(numbers, vec![2, 3]);
    assert!(!numbers.contains(&1), "target PIR must not suggest itself");
    assert!(suggestions.iter().all(|suggestion| suggestion.score <= 100));
}

#[test]
fn suggest_related_pirs_is_deterministic_for_shuffled_input_and_caps_limit() {
    let target = pir(1, "MCP metrics omitted", "metrics missing", &["mcp"]);
    let p2 = pir(2, "MCP metrics gap", "metrics missing", &["mcp"]);
    let p3 = pir(3, "MCP metrics gap", "metrics missing", &["mcp"]);
    let p4 = pir(4, "MCP metrics gap", "metrics missing", &["mcp"]);

    let options = RelatedPirOptions::new(Some(99), Some(1));
    assert_eq!(options.limit, 20, "limit must be capped at 20");

    let first = suggest_related_pirs(
        &[target.clone(), p4.clone(), p2.clone(), p3.clone()],
        1,
        options.clone(),
    )
    .expect("first related suggestions");
    let second = suggest_related_pirs(&[p3, target, p2, p4], 1, options)
        .expect("second related suggestions");

    let first_numbers: Vec<u32> = first.iter().map(|suggestion| suggestion.number).collect();
    let second_numbers: Vec<u32> = second.iter().map(|suggestion| suggestion.number).collect();
    assert_eq!(first_numbers, vec![2, 3, 4]);
    assert_eq!(first_numbers, second_numbers);
}

#[test]
fn related_suggestions_serialize_without_forbidden_fields() {
    let target = pir(
        1,
        "MCP metrics omitted",
        "Agent cannot retrieve incident metrics over MCP",
        &["mcp", "metrics", "agent"],
    );
    let mut candidate = pir(
        2,
        "MCP metrics omitted from tool list",
        "The problem statement has words that must not be returned as shared terms",
        &[
            "mcp",
            "metrics",
            "agent",
            "workflow",
            "reporting",
            "privacy",
        ],
    );
    candidate.root_cause = Some("private implementation detail".into());

    let suggestions = suggest_related_pirs(
        &[target, candidate],
        1,
        RelatedPirOptions::new(Some(5), Some(1)),
    )
    .expect("related suggestions");
    let value = serde_json::to_value(&suggestions[0]).expect("serialize suggestion");
    let object = value.as_object().expect("suggestion object");

    for key in [
        "number",
        "title",
        "status",
        "severity",
        "incident_type",
        "tags",
        "score",
        "signals",
    ] {
        assert!(object.contains_key(key), "missing expected key {key}");
    }

    for forbidden in [
        "problem_statement",
        "root_cause",
        "timeline",
        "five_whys",
        "actions",
        "shared_terms",
        "body_excerpt",
    ] {
        assert!(
            !object.contains_key(forbidden),
            "forbidden key {forbidden} leaked"
        );
    }

    let shared_tags = object["signals"]["shared_tags"].as_array().unwrap();
    assert!(shared_tags.len() <= 5);
    assert!(shared_tags.iter().any(|tag| tag == "mcp"));
    assert!(shared_tags.iter().any(|tag| tag == "metrics"));
    assert!(shared_tags.iter().any(|tag| tag == "agent"));
    assert!(matches!(
        object["signals"]["shared_token_count"],
        Value::Number(_)
    ));
}

#[test]
fn missing_target_returns_error() {
    let pirs = vec![pir(1, "Only PIR", "problem", &["mcp"])];
    let err = suggest_related_pirs(&pirs, 99, RelatedPirOptions::default())
        .expect_err("missing target should be an error");
    assert_eq!(err, RelatedPirError::TargetNotFound(99));
}

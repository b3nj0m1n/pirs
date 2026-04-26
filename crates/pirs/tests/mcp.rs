//! MCP server integration tests.
//!
//! Spawns `pirs mcp serve` as a subprocess, sends newline-delimited JSON-RPC
//! messages over stdio, and asserts on the responses.

use assert_cmd::cargo::CommandCargoExt;
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::Duration;

fn rpc_request(id: u32, method: &str, params: Value) -> String {
    let mut s = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    })
    .to_string();
    s.push('\n');
    s
}

fn rpc_notification(method: &str, params: Value) -> String {
    let mut s = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
    })
    .to_string();
    s.push('\n');
    s
}

fn init_repo(temp: &assert_fs::TempDir) {
    assert_cmd::Command::cargo_bin("pirs")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();
}

/// Drive the MCP server with a sequence of stdin lines, then collect responses
/// keyed by request id. Times out the whole exchange.
fn drive_server(temp: &assert_fs::TempDir, requests: &[String]) -> Vec<Value> {
    let mut child = Command::cargo_bin("pirs")
        .unwrap()
        .current_dir(temp.path())
        .args(["mcp", "serve"])
        .env("RUST_LOG", "warn")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn pirs mcp serve");

    let mut stdin = child.stdin.take().unwrap();
    for line in requests {
        stdin.write_all(line.as_bytes()).expect("write stdin");
    }
    drop(stdin); // EOF terminates the server

    let stdout = child.stdout.take().unwrap();
    let reader_handle = std::thread::spawn(move || {
        let mut out = Vec::new();
        for line in BufReader::new(stdout).lines() {
            match line {
                Ok(l) if !l.trim().is_empty() => match serde_json::from_str::<Value>(&l) {
                    Ok(v) => out.push(v),
                    Err(e) => panic!("non-JSON line on stdout (parse error: {e}): {l:?}"),
                },
                Ok(_) => continue,
                Err(_) => break,
            }
        }
        out
    });

    // Drain stderr concurrently so the child can't block on a full pipe.
    let stderr = child.stderr.take().unwrap();
    let stderr_handle = std::thread::spawn(move || {
        let mut buf = String::new();
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            buf.push_str(&line);
            buf.push('\n');
        }
        buf
    });

    // Wait for the child with a timeout fallback.
    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if start.elapsed() > Duration::from_secs(15) => {
                let _ = child.kill();
                panic!("pirs mcp serve did not terminate within 15s");
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(e) => panic!("wait failed: {e}"),
        }
    }
    let _stderr_buf = stderr_handle.join().expect("stderr thread");
    reader_handle.join().expect("reader thread")
}

/// Find the response matching a given request id; ignore notifications.
fn find_response(responses: &[Value], id: u32) -> Option<&Value> {
    responses
        .iter()
        .find(|v| v.get("id").and_then(|i| i.as_u64()) == Some(id as u64))
}

fn initialize_seq() -> Vec<String> {
    vec![
        rpc_request(
            1,
            "initialize",
            json!({
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": { "name": "pirs-test", "version": "0.0.0" }
            }),
        ),
        rpc_notification("notifications/initialized", json!({})),
    ]
}

fn extract_tool_text(result: &Value) -> Option<String> {
    result
        .get("result")
        .and_then(|r| r.get("content"))
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|c| c.get("text"))
        .and_then(|t| t.as_str())
        .map(|s| s.to_string())
}

#[test]
fn mcp_tools_list_advertises_read_and_write_tools() {
    let temp = assert_fs::TempDir::new().unwrap();
    init_repo(&temp);

    let mut reqs = initialize_seq();
    reqs.push(rpc_request(2, "tools/list", json!({})));

    let responses = drive_server(&temp, &reqs);
    let resp = find_response(&responses, 2).expect("tools/list response");
    let tools = resp
        .get("result")
        .and_then(|r| r.get("tools"))
        .and_then(|t| t.as_array())
        .expect("tools array");
    let names: Vec<String> = tools
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()).map(String::from))
        .collect();

    // REQ-MCP-003 read tools.
    for n in [
        "list_pirs",
        "get_pir",
        "search_pirs",
        "get_open_actions",
        "get_repository_info",
        "validate_pir",
        "get_incident_metrics",
        "suggest_related_pirs",
    ] {
        assert!(
            names.iter().any(|x| x == n),
            "missing read tool {n}: {names:?}"
        );
    }
    // REQ-MCP-004 write tools.
    for n in [
        "create_pir",
        "append_timeline_event",
        "update_status",
        "add_why",
        "add_action",
        "update_action",
        "link_evidence",
    ] {
        assert!(
            names.iter().any(|x| x == n),
            "missing write tool {n}: {names:?}"
        );
    }
}

#[test]
fn mcp_create_pir_then_list_pirs() {
    let temp = assert_fs::TempDir::new().unwrap();
    init_repo(&temp);

    let mut reqs = initialize_seq();
    reqs.push(rpc_request(
        2,
        "tools/call",
        json!({
            "name": "create_pir",
            "arguments": {
                "title": "Failing cargo test after parser change",
                "problem_statement": "cargo test failed after parser metadata update",
                "incident_type": "development",
                "severity": "medium",
                "agent": "GitHub Copilot",
            }
        }),
    ));
    reqs.push(rpc_request(
        3,
        "tools/call",
        json!({ "name": "list_pirs", "arguments": {} }),
    ));

    let responses = drive_server(&temp, &reqs);

    let create_resp = find_response(&responses, 2).expect("create_pir response");
    let create_text = extract_tool_text(create_resp).expect("create text");
    let create_value: Value = serde_json::from_str(&create_text).unwrap();
    assert_eq!(create_value["number"], json!(1));

    let list_resp = find_response(&responses, 3).expect("list_pirs response");
    let list_text = extract_tool_text(list_resp).expect("list text");
    let list_value: Value = serde_json::from_str(&list_text).unwrap();
    assert_eq!(list_value["count"], json!(1));
    assert_eq!(
        list_value["pirs"][0]["title"],
        json!("Failing cargo test after parser change")
    );

    // The PIR file actually exists on disk (REQ-MCP-002 — per-call repo writes through).
    assert!(
        temp.path()
            .join("doc/pir/0001-failing-cargo-test-after-parser-change.md")
            .is_file()
    );
    let entries: Vec<_> = std::fs::read_dir(temp.path().join("doc/pir"))
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"))
        .collect();
    assert_eq!(entries.len(), 1);
}

#[test]
fn mcp_full_lifecycle_resolved_with_actions_and_whys() {
    let temp = assert_fs::TempDir::new().unwrap();
    init_repo(&temp);

    let mut reqs = initialize_seq();
    reqs.push(rpc_request(
        2,
        "tools/call",
        json!({
            "name": "create_pir",
            "arguments": {
                "title": "Outage",
                "problem_statement": "API returned 500s",
                "agent": "GitHub Copilot",
            }
        }),
    ));
    reqs.push(rpc_request(
        3,
        "tools/call",
        json!({
            "name": "append_timeline_event",
            "arguments": {
                "pir": 1,
                "actor": "GitHub Copilot",
                "event_type": "investigated",
                "message": "found stale cache"
            }
        }),
    ));
    reqs.push(rpc_request(
        4,
        "tools/call",
        json!({
            "name": "add_why",
            "arguments": {
                "pir": 1,
                "question": "why?",
                "answer": "stale cache",
                "as_root_cause": true
            }
        }),
    ));
    reqs.push(rpc_request(
        5,
        "tools/call",
        json!({
            "name": "add_action",
            "arguments": {
                "pir": 1,
                "description": "Add regression test",
                "owner": "GitHub Copilot",
                "owner_type": "agent",
                "due": "2026-12-31"
            }
        }),
    ));
    reqs.push(rpc_request(
        6,
        "tools/call",
        json!({
            "name": "update_status",
            "arguments": { "pir": 1, "status": "resolved", "now": true }
        }),
    ));
    reqs.push(rpc_request(
        7,
        "tools/call",
        json!({ "name": "validate_pir", "arguments": { "pir": 1 } }),
    ));

    let responses = drive_server(&temp, &reqs);

    let add_action_text = extract_tool_text(find_response(&responses, 5).unwrap()).unwrap();
    let add_action_value: Value = serde_json::from_str(&add_action_text).unwrap();
    assert_eq!(add_action_value["action_id"], json!("ACT-001"));

    let validate_text = extract_tool_text(find_response(&responses, 7).unwrap()).unwrap();
    let validate_value: Value = serde_json::from_str(&validate_text).unwrap();
    assert_eq!(validate_value["ready_for_review"], json!(true));
}

#[test]
fn mcp_get_repository_info_returns_resolved_paths() {
    let temp = assert_fs::TempDir::new().unwrap();
    init_repo(&temp);

    let mut reqs = initialize_seq();
    reqs.push(rpc_request(
        2,
        "tools/call",
        json!({ "name": "get_repository_info", "arguments": {} }),
    ));

    let responses = drive_server(&temp, &reqs);
    let text = extract_tool_text(find_response(&responses, 2).unwrap()).unwrap();
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["pir_dir"], json!("doc/pir"));
    assert_eq!(v["total_pirs"], json!(0));
}

#[test]
fn mcp_get_incident_metrics_returns_filtered_metrics_and_text() {
    let temp = assert_fs::TempDir::new().unwrap();
    init_repo(&temp);

    let mut reqs = initialize_seq();
    reqs.push(rpc_request(
        2,
        "tools/call",
        json!({
            "name": "create_pir",
            "arguments": {
                "title": "Metrics outage",
                "problem_statement": "MCP metrics were unavailable",
                "severity": "high",
                "incident_type": "development",
                "tags": ["mcp", "metrics"]
            }
        }),
    ));
    reqs.push(rpc_request(
        3,
        "tools/call",
        json!({
            "name": "create_pir",
            "arguments": {
                "title": "Template issue",
                "problem_statement": "Template heading typo",
                "severity": "low",
                "incident_type": "process",
                "tags": ["docs"]
            }
        }),
    ));
    reqs.push(rpc_request(
        4,
        "tools/call",
        json!({
            "name": "get_incident_metrics",
            "arguments": { "severity": "high", "include_text": true }
        }),
    ));

    let responses = drive_server(&temp, &reqs);
    let text = extract_tool_text(find_response(&responses, 4).unwrap()).unwrap();
    let v: Value = serde_json::from_str(&text).unwrap();

    assert_eq!(v["filters"]["severity"], json!("high"));
    assert_eq!(v["metrics"]["total"], json!(1));
    assert_eq!(v["metrics"]["by_severity"]["High"], json!(1));
    assert!(v["summary_text"].as_str().unwrap().contains("Incidents: 1"));
}

#[test]
fn mcp_suggest_related_pirs_returns_ranked_privacy_safe_suggestions() {
    let temp = assert_fs::TempDir::new().unwrap();
    init_repo(&temp);

    let mut reqs = initialize_seq();
    for (id, title, problem, tags) in [
        (
            2,
            "MCP metrics omitted",
            "Agent cannot retrieve incident metrics over MCP",
            json!(["mcp", "metrics"]),
        ),
        (
            3,
            "MCP incident metrics missing",
            "MCP client needs incident metrics for agent workflow",
            json!(["mcp", "metrics"]),
        ),
        (
            4,
            "MCP related search unclear",
            "Agent needs related incident suggestions",
            json!(["mcp"]),
        ),
    ] {
        reqs.push(rpc_request(
            id,
            "tools/call",
            json!({
                "name": "create_pir",
                "arguments": {
                    "title": title,
                    "problem_statement": problem,
                    "severity": "high",
                    "incident_type": "development",
                    "tags": tags
                }
            }),
        ));
    }
    reqs.push(rpc_request(
        5,
        "tools/call",
        json!({
            "name": "suggest_related_pirs",
            "arguments": { "pir": 1, "limit": 2, "min_score": 1 }
        }),
    ));

    let responses = drive_server(&temp, &reqs);
    let text = extract_tool_text(find_response(&responses, 5).unwrap()).unwrap();
    let v: Value = serde_json::from_str(&text).unwrap();
    let suggestions = v["suggestions"].as_array().unwrap();

    assert_eq!(v["count"], json!(2));
    assert_eq!(suggestions[0]["number"], json!(2));
    assert!(suggestions[0]["signals"]["shared_token_count"].is_number());
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
            suggestions[0].get(forbidden).is_none(),
            "forbidden key {forbidden} leaked"
        );
    }
}

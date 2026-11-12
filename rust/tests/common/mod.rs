//! Shared setup for the rule tests in `rust/tests/R-n/` (D-19). Each target
//! includes this file with `#[path = "../common/mod.rs"] mod common;` and uses
//! only the helpers it needs.
#![allow(dead_code)]

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use serde_json::Value;
use tempfile::TempDir;

use pjpd::server::{ListTasksParams, MarkTaskParams, PjpdServer, PutIdeaParams, PutTaskParams};

/// A server rooted at a fresh temp directory; keep the TempDir alive.
pub fn server() -> (TempDir, PjpdServer) {
    let dir = TempDir::new().unwrap();
    let server = PjpdServer::new(dir.path().to_path_buf());
    (dir, server)
}

pub fn response_json(result: &CallToolResult) -> Value {
    let text = &result.content[0].as_text().expect("text content").text;
    serde_json::from_str(text).expect("valid JSON response")
}

pub fn assert_success(response: &Value) -> Value {
    assert_eq!(
        response["success"],
        Value::Bool(true),
        "expected success, got: {response}"
    );
    response["result"].clone()
}

pub fn assert_failure(response: &Value) -> String {
    assert_eq!(
        response["success"],
        Value::Bool(false),
        "expected failure, got: {response}"
    );
    response["error"].as_str().unwrap().to_string()
}

/// Create a task through `pjpd_put_task` and return its ID.
pub async fn create_task(
    server: &PjpdServer,
    description: &str,
    tag: &str,
    priority: i64,
) -> String {
    let result = server
        .pjpd_put_task(Parameters(PutTaskParams {
            description: description.to_string(),
            tag: Some(tag.to_string()),
            id: None,
            priority: Some(priority),
            verbose: false,
        }))
        .await;
    let body = assert_success(&response_json(&result));
    body["id"].as_str().unwrap().to_string()
}

/// Create an idea through `pjpd_put_idea` and return its ID.
pub async fn create_idea(server: &PjpdServer, description: &str, tag: &str, score: i64) -> String {
    let result = server
        .pjpd_put_idea(Parameters(PutIdeaParams {
            score,
            description: description.to_string(),
            tag: Some(tag.to_string()),
            id: None,
            verbose: false,
        }))
        .await;
    let body = assert_success(&response_json(&result));
    body["id"].as_str().unwrap().to_string()
}

/// Call `pjpd_mark_task` and return the parsed response.
pub async fn mark_tasks(server: &PjpdServer, task_ids: &[&str], state: &str) -> Value {
    let result = server
        .pjpd_mark_task(Parameters(MarkTaskParams {
            task_ids: task_ids.iter().map(|id| id.to_string()).collect(),
            state: state.to_string(),
            verbose: false,
        }))
        .await;
    response_json(&result)
}

/// Call `pjpd_list_tasks` with a status filter and return the success result.
pub async fn list_tasks(server: &PjpdServer, status: &str) -> Value {
    let result = server
        .pjpd_list_tasks(Parameters(ListTasksParams {
            count: 20,
            status: status.to_string(),
            verbose: false,
        }))
        .await;
    assert_success(&response_json(&result))
}

/// Delete `pjpd/bak/` so a test can count the backups one call makes.
pub fn clear_backups(dir: &TempDir) {
    let bak_dir = dir.path().join("pjpd/bak");
    if bak_dir.exists() {
        std::fs::remove_dir_all(bak_dir).unwrap();
    }
}

/// Contents of every `pjpd/bak/` backup of the file with this stem
/// (e.g. "tasks" for tasks.txt).
pub fn backups_of(dir: &TempDir, stem: &str) -> Vec<String> {
    let bak_dir = dir.path().join("pjpd/bak");
    if !bak_dir.exists() {
        return Vec::new();
    }
    std::fs::read_dir(bak_dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(&format!("{stem}.")))
        })
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect()
}

/// Every file under `pjpd/` (backups included), path relative to `pjpd/`
/// mapped to contents, so a test can assert that a call changed nothing.
pub fn snapshot(dir: &TempDir) -> std::collections::BTreeMap<String, String> {
    fn walk(root: &std::path::Path, path: &std::path::Path, out: &mut Vec<(String, String)>) {
        if !path.exists() {
            return;
        }
        for entry in std::fs::read_dir(path).unwrap() {
            let p = entry.unwrap().path();
            if p.is_dir() {
                walk(root, &p, out);
            } else {
                let rel = p.strip_prefix(root).unwrap().to_string_lossy().into_owned();
                out.push((rel, std::fs::read_to_string(&p).unwrap()));
            }
        }
    }
    let root = dir.path().join("pjpd");
    let mut files = Vec::new();
    walk(&root, &root, &mut files);
    files.into_iter().collect()
}

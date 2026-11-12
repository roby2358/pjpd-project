//! Shared setup for the rule tests in `rust/tests/R-n/` (D-19). Each target
//! includes this file with `#[path = "../common/mod.rs"] mod common;` and uses
//! only the helpers it needs.
#![allow(dead_code)]

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use serde_json::Value;
use tempfile::TempDir;

use pjpd::server::{ListTasksParams, MarkTaskParams, PjpdServer, PutTaskParams};

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
        }))
        .await;
    assert_success(&response_json(&result))
}

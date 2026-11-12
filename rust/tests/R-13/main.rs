//! R-13: `pjpd_put_task` with `tag` creates a task with a generated ID, the
//! given priority or DEFAULT_NEW_TASK_PRIORITY, status `ToDo`, creating
//! `pjpd/tasks.txt` if missing.

#[path = "../common/mod.rs"]
mod common;

use common::{assert_success, response_json, server};
use pjpd::server::{DEFAULT_NEW_TASK_PRIORITY, PutTaskParams};
use rmcp::handler::server::wrapper::Parameters;

// R-13: a created task has a tag-based ID, the given priority, ToDo, and the
// tasks file now exists.
#[tokio::test]
async fn put_task_creates_task_with_metadata() {
    let (dir, server) = server();
    let result = server
        .pjpd_put_task(Parameters(PutTaskParams {
            description: "Write the docs".to_string(),
            tag: Some("docs".to_string()),
            id: None,
            priority: Some(7),
            verbose: false,
        }))
        .await;
    let body = assert_success(&response_json(&result));

    assert!(body["id"].as_str().unwrap().starts_with("docs-"));
    assert_eq!(body["priority"], 7);
    assert_eq!(body["status"], "ToDo");
    assert_eq!(body["message"], "Task created successfully");
    assert!(dir.path().join("pjpd/tasks.txt").exists());
}

// R-13: without a priority the new task gets DEFAULT_NEW_TASK_PRIORITY.
#[tokio::test]
async fn put_task_create_without_priority_gets_the_default() {
    let (_dir, server) = server();
    let result = server
        .pjpd_put_task(Parameters(PutTaskParams {
            description: "default priority".to_string(),
            tag: Some("defp".to_string()),
            id: None,
            priority: None,
            verbose: false,
        }))
        .await;
    let body = assert_success(&response_json(&result));
    assert_eq!(body["priority"], DEFAULT_NEW_TASK_PRIORITY);
}

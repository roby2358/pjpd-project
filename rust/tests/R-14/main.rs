//! R-14: `pjpd_put_task` with `id` replaces the description, replaces priority
//! only when given, and fails for an unknown ID naming `pjpd_list_tasks`.

#[path = "../common/mod.rs"]
mod common;

use common::{assert_failure, assert_success, create_task, response_json, server};
use pjpd::server::PutTaskParams;
use rmcp::handler::server::wrapper::Parameters;

// R-14: description and a given priority are replaced.
#[tokio::test]
async fn put_task_updates_existing_task() {
    let (_dir, server) = server();
    let id = create_task(&server, "original", "fix", 3).await;

    let result = server
        .pjpd_put_task(Parameters(PutTaskParams {
            description: "updated text".to_string(),
            tag: None,
            id: Some(id.clone()),
            priority: Some(9),
            verbose: true,
        }))
        .await;
    let body = assert_success(&response_json(&result));
    assert_eq!(body["description"], "updated text");
    assert_eq!(body["priority"], 9);
    assert_eq!(body["message"], format!("Task '{id}' updated successfully"));
}

// R-14: an omitted priority keeps the existing one.
#[tokio::test]
async fn put_task_update_without_priority_preserves_existing() {
    let (_dir, server) = server();
    let id = create_task(&server, "keep my priority", "keep", 33).await;

    let result = server
        .pjpd_put_task(Parameters(PutTaskParams {
            description: "new words, same priority".to_string(),
            tag: None,
            id: Some(id.clone()),
            priority: None,
            verbose: true,
        }))
        .await;
    let body = assert_success(&response_json(&result));
    assert_eq!(body["description"], "new words, same priority");
    assert_eq!(body["priority"], 33);
}

// R-14: an unknown ID fails with a not-found message.
#[tokio::test]
async fn put_task_unknown_id_fails_gracefully() {
    let (_dir, server) = server();
    let result = server
        .pjpd_put_task(Parameters(PutTaskParams {
            description: "x".to_string(),
            tag: None,
            id: Some("ghost-ab29".to_string()),
            priority: Some(50),
            verbose: false,
        }))
        .await;
    let error = assert_failure(&response_json(&result));
    assert!(
        error.contains("Task 'ghost-ab29' not found"),
        "got: {error}"
    );
}

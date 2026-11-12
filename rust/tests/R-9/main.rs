//! R-9: stamping sets `created` once, only if missing, and refreshes `updated`;
//! both are ISO-8601 UTC.

#[path = "../common/mod.rs"]
mod common;

use common::{assert_success, response_json, server};
use pjpd::server::PutTaskParams;
use pjpd::tasks::{TaskPatch, TaskStore};
use rmcp::handler::server::wrapper::Parameters;
use tempfile::TempDir;

// R-9: a patch keeps `created` and refreshes `updated`.
#[test]
fn update_preserves_created_and_refreshes_updated() {
    let dir = TempDir::new().unwrap();
    let store = TaskStore::new(dir.path());

    let task = store.add("stamped", 5, "time").unwrap();
    let created = task.created.clone().unwrap();
    assert_eq!(task.updated.as_deref(), Some(created.as_str()));

    let describe = TaskPatch {
        description: Some("new text".to_string()),
        priority: None,
        status: None,
    };
    let updated = store.update(&task.id, &describe).unwrap().unwrap();
    assert_eq!(updated.created.as_deref(), Some(created.as_str()));
    assert!(updated.updated.is_some());
}

// R-9: a task created through the tool carries a UTC (`Z`) `created` stamp.
#[tokio::test]
async fn put_task_create_stamps_created_in_utc() {
    let (_dir, server) = server();
    let result = server
        .pjpd_put_task(Parameters(PutTaskParams {
            description: "Write the docs".to_string(),
            tag: Some("docs".to_string()),
            id: None,
            priority: Some(7),
            verbose: true,
        }))
        .await;
    let body = assert_success(&response_json(&result));
    assert!(body["created"].as_str().unwrap().ends_with('Z'));
}
